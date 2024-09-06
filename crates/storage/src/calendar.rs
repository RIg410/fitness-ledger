use bson::to_document;
use chrono::{Duration, Utc, Weekday};
use eyre::Result;
use model::{day::Day, ids::DayId, training::Training};
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{FindOneOptions, IndexOptions, UpdateOptions},
    ClientSession, Collection, Database, IndexModel,
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

    pub async fn get_my_trainings(
        &self,
        session: &mut ClientSession,
        id: ObjectId,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Training>, eyre::Error> {
        let now = Utc::now();
        let day_id = DayId::from(now);
        let filter = doc! {
          "$and": [
            {
              "$or": [
                { "training.instructor": id },
                { "training.clients": { "$elemMatch": { "$eq": id } } }
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
                if training.instructor == id || training.clients.contains(&id) {
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

                let day = prev_day.copy(id);

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

    pub async fn update_day(
        &self,
        session: &mut ClientSession,
        day: &Day,
    ) -> Result<(), eyre::Error> {
        self.days
            .replace_one(doc! { "_id": day.id }, day)
            .session(&mut *session)
            .await?;
        Ok(())
    }
}
