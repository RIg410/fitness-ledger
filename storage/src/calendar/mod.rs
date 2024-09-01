pub mod model;
use chrono::{DateTime, Local, TimeZone, Weekday};
use eyre::Result;
use futures_util::StreamExt as _;
use model::Week;
use mongodb::{
    bson::{doc, oid::ObjectId, to_bson},
    Collection, Database,
};
use std::sync::Arc;

use crate::training::model::Training;

const COLLECTION: &str = "schedule";

#[derive(Clone)]
pub struct CalendarStore {
    pub(crate) schedule: Arc<Collection<Week>>,
}

impl CalendarStore {
    pub(crate) fn new(db: &Database) -> Self {
        let schedule = db.collection(COLLECTION);

        CalendarStore {
            schedule: Arc::new(schedule),
        }
    }

    pub async fn get_week(&self, date_time: DateTime<Local>) -> Result<Week> {
        let week_id = week_id(date_time).ok_or(eyre::eyre!("Invalid date"))?;
        let filter = doc! { "id": to_bson(&week_id)? };
        let week = self.schedule.find_one(filter).await?;

        match week {
            Some(week) => Ok(week),
            None => {
                if week_id + chrono::Duration::days(28) < chrono::Local::now() {
                    return Err(eyre::eyre!("Week is too far in the past:{}", week_id));
                }
                let week = Week::new(week_id);
                self.schedule.insert_one(week.clone()).await?;
                Ok(week)
            }
        }
    }

    pub async fn week_cursor(&self, date_time: DateTime<Local>) -> Result<mongodb::Cursor<Week>> {
        Ok(self
            .schedule
            .find(doc! { "id": { "$gt": to_bson(&date_time)? } })
            .await?)
    }

    pub async fn update_week(&self, mut week: Week) -> Result<()> {
        let filter = doc! { "id": to_bson(&week.id)? };
        week.canonize();

        self.schedule.replace_one(filter, week).await?;
        Ok(())
    }

    pub async fn get_my_trainings(
        &self,
        id: ObjectId,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Training>, eyre::Error> {
        let filter = doc! {
          "$and": [
            {
              "$or": [
                { "days.training.instructor": id },
                { "days.training.clients": { "$elemMatch": { "$eq": id } } }
              ]
            },
            { "id": { "$gt": to_bson(&Local::now())? } }
          ]
        };

        let mut cursor = self.schedule.find(filter).await?;
        let mut trainings = Vec::new();
        let mut skiped = 0;
        let now = Local::now();
        while let Some(week) = cursor.next().await {
            let week = week?;
            for day in week.days {
                for training in day.training {
                    if training.start_at < now {
                        continue;
                    }
                    if training.instructor == id || training.clients.contains(&id) {
                        if skiped <= offset {
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
        }

        Ok(trainings)
    }
}

pub fn week_id(date_time: DateTime<Local>) -> Option<DateTime<Local>> {
    let date = date_time
        .date_naive()
        .week(Weekday::Mon)
        .first_day()
        .and_hms_opt(0, 0, 0)?;
    Local.from_local_datetime(&date).single()
}

pub fn day_id(date_time: DateTime<Local>) -> Option<DateTime<Local>> {
    let date = date_time.date_naive().and_hms_opt(0, 0, 0)?;
    Local.from_local_datetime(&date).single()
}
