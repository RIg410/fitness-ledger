use bson::to_document;
use chrono::{DateTime, Duration, Utc, Weekday};
use eyre::Result;
use log::info;
use model::{day::Day, ids::DayId, training::Training};
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{FindOneOptions, IndexOptions, UpdateOptions},
    ClientSession, Collection, Database, IndexModel, SessionCursor,
};
use std::sync::Arc;

const COLLECTION: &str = "days";

#[derive(Clone)]
pub struct CalendarStore {
    pub(crate) days: Arc<Collection<Day>>,
}

impl CalendarStore {
    pub(crate) async fn new(db: &Database) -> Result<Self> {
        let days = db.collection(COLLECTION);
        let index = IndexModel::builder()
            .keys(doc! { "date_time": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        days.create_index(index).await?;
        Ok(CalendarStore {
            days: Arc::new(days),
        })
    }

    pub async fn cursor(
        &self,
        session: &mut ClientSession,
        from: DayId,
        week_day: Weekday,
    ) -> Result<mongodb::SessionCursor<Day>> {
        let filter = doc! {
            "weekday": week_day.to_string(),
            "date_time": { "$gt": from.id() },
        };
        Ok(self.days.find(filter).session(&mut *session).await?)
    }

    pub async fn set_cancel_flag(
        &self,
        session: &mut ClientSession,
        start_at: DateTime<Utc>,
        flag: bool,
    ) -> Result<(), eyre::Error> {
        info!("Set cancel flag: {:?} {}", start_at, flag);
        let filter = doc! { "training.start_at": start_at };
        let update = doc! { "$set": { "training.$.is_canceled": flag }, "$inc": { "version": 1 } };
        let result = self
            .days
            .update_one(filter, update)
            .session(&mut *session)
            .await?;

        if result.modified_count == 0 {
            return Err(eyre::eyre!("Training not found"));
        }

        Ok(())
    }

    pub async fn find_trainings(
        &self,
        session: &mut ClientSession,
        user_id: ObjectId,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Training>, eyre::Error> {
        let now = Utc::now();
        let day_id = DayId::from(now);
        let filter = doc! {
          "$and": [
            {
              "$or": [
                { "training.instructor": user_id },
                { "training.clients": { "$elemMatch": { "$eq": user_id } } }
              ]
            },
            { "date_time": { "$gte":  day_id.id()} }
          ]
        };

        let mut cursor = self.days.find(filter).session(&mut *session).await?;
        let mut trainings = Vec::new();
        let mut skiped = 0;
        while let Some(day) = cursor.next(&mut *session).await {
            let day = day?;
            for training in day.training {
                if training.start_at + Duration::minutes(training.duration_min as i64) < now {
                    continue;
                }
                if training.instructor == user_id || training.clients.contains(&user_id) {
                    if skiped < offset {
                        skiped += 1;
                        continue;
                    }
                    if trainings.len() >= limit {
                        return Ok(trainings);
                    }
                    trainings.push(training);
                }
            }
        }
        Ok(trainings)
    }

    pub async fn get_day(
        &self,
        session: &mut ClientSession,
        id: DayId,
    ) -> Result<Day, eyre::Error> {
        let day = self
            .days
            .find_one(doc! { "date_time": id.id() })
            .session(&mut *session)
            .await?;
        match day {
            Some(day) => return Ok(day),
            None => {
                let now = Utc::now();
                if id.id() < now - chrono::Duration::days(10) {
                    return Err(eyre::eyre!("Day is too far in the past:{:?}", id));
                }
                if now + chrono::Duration::days(365 * 2) < id.id() {
                    return Err(eyre::eyre!("Day is too far in the future:{:?}", id));
                }

                let filter = doc! {
                    "weekday": id.week_day().to_string(),
                    "date_time": { "$lt": id.id() },
                };

                let find_options = FindOneOptions::builder()
                    .sort(doc! { "date_time": -1 })
                    .build();

                let prev_day = self
                    .days
                    .find_one(filter)
                    .with_options(find_options)
                    .session(&mut *session)
                    .await?
                    .unwrap_or(Day::new(id));

                let day = Day::copy_day(id, prev_day);

                self.days
                    .update_one(
                        doc! { "date_time": day.day_date() },
                        doc! { "$setOnInsert": to_document(&day)? },
                    )
                    .session(&mut *session)
                    .with_options(UpdateOptions::builder().upsert(true).build())
                    .await?;
                return Ok(day);
            }
        }
    }

    pub async fn delete_training(
        &self,
        session: &mut ClientSession,
        start_at: DateTime<Utc>,
    ) -> std::result::Result<(), eyre::Error> {
        info!("Delete training: {:?}", start_at);
        let filter = doc! { "training.start_at": start_at };
        let update =
            doc! { "$pull": { "training": { "start_at": start_at } }, "$inc": { "version": 1 } };
        self.days
            .update_one(filter, update)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn week_days_after(
        &self,
        session: &mut ClientSession,
        day: DayId,
    ) -> Result<SessionCursor<Day>> {
        let filter = doc! {
            "date_time": { "$gt": day.id() },
            "weekday": day.week_day().to_string(),
        };
        Ok(self.days.find(filter).session(&mut *session).await?)
    }

    pub async fn add_training(
        &self,
        session: &mut ClientSession,
        training: &Training,
    ) -> Result<(), eyre::Error> {
        info!("Add training: {:?}", training);
        let filter = doc! { "date_time": training.day_id().id() };
        let update = doc! {
            "$push": { "training": to_document(training)? },
            "$inc": { "version": 1 }
        };
        self.days
            .update_one(filter, update)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn sign_up(
        &self,
        session: &mut ClientSession,
        start_at: DateTime<Utc>,
        user_id: ObjectId,
    ) -> Result<(), eyre::Error> {
        info!("Sign up: {:?} {}", start_at, user_id);
        let filter = doc! { "training.start_at": start_at };
        let update = doc! {
            "$addToSet": { "training.$.clients": user_id },
            "$inc": { "version": 1 }
        };
        let result = self
            .days
            .update_one(filter, update)
            .session(&mut *session)
            .await?;

        if result.modified_count != 1 {
            return Err(eyre::eyre!("Training not found"));
        }
        Ok(())
    }

    pub async fn sign_out(
        &self,
        session: &mut ClientSession,
        start_at: DateTime<Utc>,
        user_id: ObjectId,
    ) -> Result<(), eyre::Error> {
        info!("Sign out: {:?} {}", start_at, user_id);
        let filter = doc! { "training.start_at": start_at };
        let update = doc! {
            "$pull": { "training.$.clients": user_id },
            "$inc": { "version": 1 }
        };
        let result = self
            .days
            .update_one(filter, update)
            .session(&mut *session)
            .await?;

        if result.modified_count != 1 {
            return Err(eyre::eyre!("Training not found"));
        }
        Ok(())
    }

    pub async fn days_to_process(
        &self,
        session: &mut ClientSession,
    ) -> Result<mongodb::SessionCursor<Day>> {
        let now = Utc::now() + Duration::minutes(5);
        let filter = doc! {
            "training.start_at": { "$lt": now },
            "training.is_finished": { "$ne": true },
        };
        Ok(self.days.find(filter).session(&mut *session).await?)
    }

    pub async fn finalized(
        &self,
        session: &mut ClientSession,
        start_at: DateTime<Utc>,
    ) -> Result<()> {
        info!("Finalized: {:?}", start_at);
        let filter = doc! { "training.start_at": start_at };
        let update = doc! {
            "$set": { "training.$.is_finished": true },
            "$inc": { "version": 1 }
        };
        let result = self
            .days
            .update_one(filter, update)
            .session(&mut *session)
            .await?;

        if result.modified_count != 1 {
            return Err(eyre::eyre!("Training not found"));
        }
        Ok(())
    }
}
