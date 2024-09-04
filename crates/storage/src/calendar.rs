use chrono::{Duration, Utc, Weekday};
use eyre::Result;
use futures_util::StreamExt as _;
use model::{day::Day, ids::DayId, training::Training};
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{FindOneOptions, IndexOptions},
    Collection, Database, IndexModel,
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

    pub async fn cursor(&self, from: DayId, week_day: Weekday) -> Result<mongodb::Cursor<Day>> {
        let filter = doc! {
            "weekday": week_day.to_string(),
            "date_time": { "$gt": from.id() },
        };
        Ok(self.days.find(filter).await?)
    }

    pub async fn get_my_trainings(
        &self,
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

        let mut cursor = self.days.find(filter).await?;
        let mut trainings = Vec::new();
        let mut skiped = 0;
        while let Some(day) = cursor.next().await {
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

    pub async fn update_day(&self, day: &Day) -> Result<(), eyre::Error> {
        let filter = doc! { "_id": day.id };
        self.days.replace_one(filter, day).await?;
        Ok(())
    }

    pub async fn get_day(&self, id: DayId) -> Result<Day, eyre::Error> {
        let day = self.days.find_one(doc! { "date_time": id.id() }).await?;
        match day {
            Some(day) => Ok(day),
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
                    .await?
                    .unwrap_or(Day::new(id));

                let day = prev_day.copy(id);
                self.days.insert_one(&day).await?;
                Ok(day)
            }
        }
    }
}
