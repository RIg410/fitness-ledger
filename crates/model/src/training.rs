use chrono::{DateTime, Datelike, Local, TimeZone as _, Timelike, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::ids::DayId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Training {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub proto_id: ObjectId,
    pub name: String,
    pub description: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub start_at: DateTime<Utc>,
    pub duration_min: u32,
    pub instructor: ObjectId,
    pub clients: Vec<ObjectId>,
    pub capacity: u32,
    pub status: TrainingStatus,
    pub is_one_time: bool,
}

impl Training {
    pub fn start_at_local(&self) -> DateTime<Local> {
        self.start_at.with_timezone(&Local)
    }

    pub fn end_at(&self) -> DateTime<Local> {
        self.start_at.with_timezone(&Local) + chrono::Duration::minutes(self.duration_min as i64)
    }

    pub fn is_full(&self) -> bool {
        self.clients.len() >= self.capacity as usize
    }

    pub fn is_open_to_signup(&self) -> bool {
        self.status == TrainingStatus::OpenToSignup
    }

    pub fn is_training_time(&self, time: DateTime<Local>) -> bool {
        self.start_at < time && time < self.end_at()
    }

    pub fn set_date(&mut self, week_date: DateTime<Local>) -> Result<(), eyre::Error> {
        self.start_at = self
            .start_at
            .with_day(week_date.day())
            .ok_or_else(|| eyre::eyre!("Invalid day"))?
            .with_year(week_date.year())
            .ok_or_else(|| eyre::eyre!("Invalid day"))?
            .with_month(week_date.month())
            .ok_or_else(|| eyre::eyre!("Invalid day"))?;
        Ok(())
    }

    pub fn start_at_on(&self, day_id: DayId) -> DateTime<Utc> {
        let new_date = day_id.local().naive_local().date();
        let start_date = self.start_at_local();
        let start_at = new_date
            .and_hms_opt(start_date.hour(), start_date.minute(), start_date.second())
            .expect("Invalid date");
        Local
            .from_local_datetime(&start_at)
            .single()
            .unwrap()
            .with_timezone(&Utc)
    }

    pub fn change_date(self, day_id: DayId) -> Training {
        let new_date = day_id.local().naive_local().date();
        let start_date = self.start_at_local();

        let start_at = new_date
            .and_hms_opt(start_date.hour(), start_date.minute(), start_date.second())
            .expect("Invalid date");
        let start_at = Local
            .from_local_datetime(&start_at)
            .single()
            .unwrap()
            .with_timezone(&Utc);

        Training {
            id: ObjectId::new(),
            proto_id: self.proto_id,
            name: self.name,
            description: self.description,
            start_at,
            duration_min: self.duration_min,
            instructor: self.instructor,
            clients: Vec::new(),
            capacity: self.capacity,
            status: TrainingStatus::OpenToSignup,
            is_one_time: self.is_one_time,
        }
    }

    pub fn day_id(&self) -> DayId {
        DayId::from(self.start_at)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TrainingStatus {
    OpenToSignup,
    ClosedToSignup,
    InProgress,
    Cancelled,
    Finished,
}
