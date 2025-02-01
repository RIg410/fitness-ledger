use bson::to_document;
use chrono::{DateTime, Duration, Local, Utc, Weekday};
use eyre::Result;
use log::info;
use model::{
    day::Day,
    ids::DayId,
    program::TrainingType,
    session::Session,
    training::{Filter, Notified, Statistics, Training, TrainingId},
};
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{FindOneOptions, IndexOptions, UpdateOptions},
    Collection, Database, IndexModel, SessionCursor,
};

const COLLECTION: &str = "days";

pub struct CalendarStore {
    pub(crate) store: Collection<Day>,
}

impl CalendarStore {
    pub(crate) async fn new(db: &Database) -> Result<Self> {
        let days = db.collection(COLLECTION);
        let index = IndexModel::builder()
            .keys(doc! { "date_time": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        days.create_index(index).await?;

        days.create_index(
            IndexModel::builder()
                .keys(doc! { "training.start_at": 1 })
                .build(),
        )
        .await?;

        days.create_index(
            IndexModel::builder()
                .keys(doc! { "training.room": 1 })
                .build(),
        )
        .await?;

        Ok(CalendarStore { store: days })
    }

    pub async fn find_range(
        &self,
        session: &mut Session,
        from: Option<DateTime<Local>>,
        to: Option<DateTime<Local>>,
    ) -> Result<SessionCursor<Day>> {
        let filter = match (from, to) {
            (Some(from), Some(to)) => doc! {
                "date_time": {
                    "$gte": from.with_timezone(&Utc),
                    "$lt": to.with_timezone(&Utc),
                }
            },
            (Some(from), None) => doc! {
                "date_time": {
                    "$gte": from.with_timezone(&Utc),
                }
            },
            (None, Some(to)) => doc! {
                "date_time": {
                    "$lt": to.with_timezone(&Utc),
                }
            },
            (None, None) => doc! {},
        };
        Ok(self.store.find(filter).session(&mut *session).await?)
    }

    pub async fn cursor(
        &self,
        session: &mut Session,
        from: DayId,
        week_day: Weekday,
    ) -> Result<mongodb::SessionCursor<Day>> {
        let filter = doc! {
            "weekday": week_day.to_string(),
            "date_time": { "$gt": from.id() },
        };
        Ok(self.store.find(filter).session(&mut *session).await?)
    }

    pub async fn set_cancel_flag(
        &self,
        session: &mut Session,
        id: TrainingId,
        flag: bool,
    ) -> Result<(), eyre::Error> {
        info!("Set cancel flag: {:?} {}", id, flag);
        let update = doc! { "$set": { "training.$.is_canceled": flag }, "$inc": { "version": 1 } };
        let result = self
            .store
            .update_one(training_filter(id), update)
            .session(&mut *session)
            .await?;

        if dbg!(result.modified_count) == 0 {
            return Err(eyre::eyre!("Training not found"));
        }

        Ok(())
    }

    pub async fn find_trainings(
        &self,
        session: &mut Session,
        filter: Filter,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Training>, eyre::Error> {
        let now = Utc::now();
        let day_id = DayId::from(now);

        let find = match filter {
            Filter::Client(id) => {
                doc! {
                  "$and": [
                        {
                            "training.clients": { "$elemMatch": { "$eq": id } }
                        },
                    { "date_time": { "$gte":  day_id.id()} }
                  ]
                }
            }
            Filter::Instructor(object_id) => {
                doc! {
                  "$and": [
                        {
                            "training.instructor": object_id
                        },
                    { "date_time": { "$gte":  day_id.id()} }
                  ]
                }
            }
            Filter::Program(object_id) => {
                doc! {
                  "$and": [
                        {
                            "training.proto_id": object_id
                        },
                    { "date_time": { "$gte":  day_id.id()} }
                  ]
                }
            }
        };
        let mut cursor = self.store.find(find).session(&mut *session).await?;

        let mut skiped = 0;
        let mut trainings = Vec::new();
        while let Some(day) = cursor.next(&mut *session).await {
            let mut day = day?;
            day.training.sort_by_key(|a| a.start_at_utc());
            for training in day.training {
                if training.start_at_utc() + Duration::minutes(training.duration_min as i64) < now {
                    continue;
                }
                if skiped < offset {
                    skiped += 1;
                    continue;
                }
                if trainings.len() >= limit {
                    return Ok(trainings);
                }

                if filter.is_match(&training) {
                    trainings.push(training);
                }
            }
        }
        Ok(trainings)
    }

    pub async fn get_day(&self, session: &mut Session, id: DayId) -> Result<Day, eyre::Error> {
        let day = self
            .store
            .find_one(doc! { "date_time": id.id() })
            .session(&mut *session)
            .await?;
        match day {
            Some(day) => Ok(day),
            None => {
                let now = Utc::now();
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
                    .store
                    .find_one(filter)
                    .with_options(find_options)
                    .session(&mut *session)
                    .await?
                    .unwrap_or(Day::new(id));

                let day = Day::copy_day(id, prev_day);

                self.store
                    .update_one(
                        doc! { "date_time": day.day_date() },
                        doc! { "$setOnInsert": to_document(&day)? },
                    )
                    .session(&mut *session)
                    .with_options(UpdateOptions::builder().upsert(true).build())
                    .await?;
                Ok(day)
            }
        }
    }

    pub async fn delete_training(
        &self,
        session: &mut Session,
        id: TrainingId,
    ) -> std::result::Result<(), eyre::Error> {
        info!("Delete training: {:?}", id);

        let update = doc! { "$pull": { "training": { "start_at": id.start_at, "room": id.room } }, "$inc": { "version": 1 } };
        let update = self.store
            .update_one(training_filter(id), update)
            .session(&mut *session)
            .await?;
        if update.modified_count != 1 {
            return Err(eyre::eyre!("Training not found"));
        }
        Ok(())
    }

    pub async fn week_days_after(
        &self,
        session: &mut Session,
        day: DayId,
    ) -> Result<SessionCursor<Day>> {
        let filter = doc! {
            "date_time": { "$gt": day.id() },
            "weekday": day.week_day().to_string(),
        };
        Ok(self.store.find(filter).session(&mut *session).await?)
    }

    pub async fn add_training(
        &self,
        session: &mut Session,
        training: &Training,
    ) -> Result<(), eyre::Error> {
        info!("Add training: {:?}", training);
        let filter = doc! { "date_time": training.day_id().id() };
        let update = doc! {
            "$push": { "training": to_document(training)? },
            "$inc": { "version": 1 }
        };
        self.store
            .update_one(filter, update)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn sign_up(
        &self,
        session: &mut Session,
        id: TrainingId,
        user_id: ObjectId,
    ) -> Result<(), eyre::Error> {
        info!("Sign up: {:?} {}", id, user_id);
        let update = doc! {
            "$addToSet": { "training.$.clients": user_id },
            "$inc": { "version": 1 }
        };
        let result = self
            .store
            .update_one(training_filter(id), update)
            .session(&mut *session)
            .await?;

        if result.modified_count != 1 {
            return Err(eyre::eyre!("Training not found"));
        }
        Ok(())
    }

    pub async fn sign_out(
        &self,
        session: &mut Session,
        id: TrainingId,
        user_id: ObjectId,
    ) -> Result<(), eyre::Error> {
        info!("Sign out: {:?} {}", id, user_id);
        let update = doc! {
            "$pull": { "training.$.clients": user_id },
            "$inc": { "version": 1 }
        };
        let result = self
            .store
            .update_one(training_filter(id), update)
            .session(&mut *session)
            .await?;

        if result.modified_count != 1 {
            return Err(eyre::eyre!("Training not found"));
        }
        Ok(())
    }

    pub async fn days_to_process(
        &self,
        session: &mut Session,
    ) -> Result<mongodb::SessionCursor<Day>> {
        let filter = doc! {
            "training": {
                "$elemMatch": { "start_at": { "$lt": Utc::now() },
                "$or": [
                    { "is_finished": { "$exists": false } },
                    { "is_finished": false }
                ]
                }
            }
        };
        Ok(self.store.find(filter).session(&mut *session).await?)
    }

    pub async fn finalized(
        &self,
        session: &mut Session,
        id: TrainingId,
        statistics: &Statistics,
    ) -> Result<()> {
        info!("Finalized: {:?}", id);
        let update = doc! {
            "$set": { "training.$.is_finished": true, "training.$.statistics": to_document(statistics)? },
            "$inc": { "version": 1 }
        };
        let result = self
            .store
            .update_one(training_filter(id), update)
            .session(&mut *session)
            .await?;

        if result.modified_count != 1 {
            return Err(eyre::eyre!("Training not found"));
        }
        Ok(())
    }

    pub async fn edit_capacity(
        &self,
        session: &mut Session,
        program_id: ObjectId,
        capacity: u32,
    ) -> Result<(), eyre::Error> {
        info!("Edit capacity: {:?} {}", program_id, capacity);
        let filter = doc! { "training.proto_id": program_id };
        let update = doc! {
            "$set": { "training.$[elem].capacity": capacity },
            "$inc": { "version": 1 }
        };
        self.store
            .update_many(filter, update)
            .array_filters([doc! { "elem.proto_id": program_id }])
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn edit_program_name(
        &self,
        session: &mut Session,
        program_id: ObjectId,
        name: String,
    ) -> Result<(), eyre::Error> {
        info!("Edit program name: {:?} {}", program_id, name);
        let filter = doc! { "training.proto_id": program_id };
        let update = doc! {
            "$set": { "training.$[elem].name": name },
            "$inc": { "version": 1 }
        };
        self.store
            .update_many(filter, update)
            .array_filters([doc! { "elem.proto_id": program_id }])
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn edit_program_description(
        &self,
        session: &mut Session,
        program_id: ObjectId,
        description: String,
    ) -> Result<(), eyre::Error> {
        info!("Edit program description: {:?} {}", program_id, description);
        let filter = doc! { "training.proto_id": program_id };
        let update = doc! {
            "$set": { "training.$[elem].description": description },
            "$inc": { "version": 1 }
        };
        self.store
            .update_many(filter, update)
            .array_filters([doc! { "elem.proto_id": program_id }])
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn find_with_program_id(
        &self,
        session: &mut Session,
        program_id: ObjectId,
    ) -> Result<SessionCursor<Day>, eyre::Error> {
        let filter = doc! { "training.proto_id": program_id };
        Ok(self.store.find(filter).session(&mut *session).await?)
    }

    pub async fn update_duration_in_day(
        &self,
        session: &mut Session,
        day_id: ObjectId,
        program_id: ObjectId,
        duration: u32,
    ) -> Result<(), eyre::Error> {
        info!("Update duration in day: {:?} {}", day_id, duration);
        let filter = doc! {
            "_id": day_id,
            "training.proto_id": program_id
        };
        let update = doc! {
            "$set": { "training.$[elem].duration_min": duration },
            "$inc": { "version": 1 }
        };
        self.store
            .update_one(filter, update)
            .array_filters([doc! { "elem.proto_id": program_id }])
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn change_name(
        &self,
        session: &mut Session,
        id: TrainingId,
        name: &str,
    ) -> Result<(), eyre::Error> {
        info!("Change name: {:?} {}", id, name);
        let update = doc! {
            "$set": { "training.$.name": name },
            "$inc": { "version": 1 }
        };
        let result = self
            .store
            .update_one(training_filter(id), update)
            .session(&mut *session)
            .await?;
        if result.modified_count != 1 {
            return Err(eyre::eyre!("Training not found"));
        }
        Ok(())
    }

    pub async fn change_couch(
        &self,
        session: &mut Session,
        id: TrainingId,
        couch_id: ObjectId,
    ) -> Result<(), eyre::Error> {
        info!("Change couch: {:?} {}", id, couch_id);
        let update = doc! {
            "$set": { "training.$.instructor": couch_id },
            "$inc": { "version": 1 }
        };
        let result = self
            .store
            .update_one(training_filter(id), update)
            .session(&mut *session)
            .await?;

        if result.modified_count != 1 {
            return Err(eyre::eyre!("Training not found"));
        }
        Ok(())
    }

    pub async fn set_training_type(
        &self,
        session: &mut Session,
        id: TrainingId,
        tp: TrainingType,
    ) -> Result<(), eyre::Error> {
        info!("Set type: {:?} -> {:?}", id, tp);
        let update = doc! {
            "$set": { "training.$.tp": to_document(&tp)? },
            "$inc": { "version": 1 }
        };
        let result = self
            .store
            .update_one(training_filter(id), update)
            .session(&mut *session)
            .await?;

        if result.modified_count != 1 {
            return Err(eyre::eyre!("Training not found"));
        }
        Ok(())
    }

    pub async fn set_keep_open(
        &self,
        session: &mut Session,
        id: TrainingId,
        keep_open: bool,
    ) -> Result<(), eyre::Error> {
        info!("Set keep open: {:?} {}", id, keep_open);
        let update =
            doc! { "$set": { "training.$.keep_open": keep_open }, "$inc": { "version": 1 } };
        let result = self
            .store
            .update_one(training_filter(id), update)
            .session(&mut *session)
            .await?;

        if result.modified_count == 0 {
            return Err(eyre::eyre!("Training not found"));
        }

        Ok(())
    }

    pub async fn notify(
        &self,
        session: &mut Session,
        id: TrainingId,
        notified: Notified,
    ) -> Result<(), eyre::Error> {
        info!("Notify: {:?}", id);
        let update = doc! {
            "$set": { "training.$.notified": to_document(&notified)? },
            "$inc": { "version": 1 }
        };
        let result = self
            .store
            .update_one(training_filter(id), update)
            .session(&mut *session)
            .await?;

        if result.modified_count != 1 {
            return Err(eyre::eyre!("Training not found"));
        }

        Ok(())
    }
}

fn training_filter(id: TrainingId) -> bson::Document {
    doc! {
        "date_time": id.day_id().id(),
        "training":{
            "$elemMatch": {
                "start_at": id.start_at,
                "room": id.room,
            }
        }
    }
}
