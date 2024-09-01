pub mod model;
use chrono::{DateTime, Local, TimeZone, Weekday};
use eyre::Result;
use model::Week;
use mongodb::{
    bson::{doc, to_bson},
    Collection, Database,
};
use std::sync::Arc;

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
