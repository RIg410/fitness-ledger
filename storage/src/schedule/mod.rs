pub mod model;
use chrono::{NaiveDate, Weekday};
use eyre::Result;
use model::Week;
use mongodb::{bson::doc, Collection, Database};
use std::sync::Arc;

use crate::date_time::Date;

const COLLECTION: &str = "schedule";

#[derive(Clone)]
pub struct ScheduleStore {
    pub(crate) schedule: Arc<Collection<Week>>,
}

impl ScheduleStore {
    pub(crate) fn new(db: &Database) -> Self {
        let schedule = db.collection(COLLECTION);

        ScheduleStore {
            schedule: Arc::new(schedule),
        }
    }

    pub async fn get_week(&self, native_data: NaiveDate) -> Result<Week> {
        let week_id = native_data.week(Weekday::Mon).first_day();
        let filter = doc! { "id": mongodb::bson::to_document(&Date::from(week_id))? };
        let week = self.schedule.find_one(filter).await?;

        match week {
            Some(week) => Ok(week),
            None => {
                if week_id + chrono::Duration::days(28) < chrono::Local::now().naive_local().date()
                {
                    return Err(eyre::eyre!("Week is too far in the past"));
                }
                let week = Week::new(week_id);
                self.schedule.insert_one(week.clone()).await?;
                Ok(week)
            }
        }
    }
}
