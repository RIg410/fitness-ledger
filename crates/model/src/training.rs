use chrono::{DateTime, Datelike, Local, Timelike as _, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{ids::DayId, program::Program, slot::Slot};

pub const CLOSE_SING_UP: u32 = 3 * 60; // 3 hours

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
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
    pub is_one_time: bool,
    #[serde(default)]
    pub is_canceled: bool,
    #[serde(default)]
    pub is_processed: bool,
}

impl Training {
    pub fn new(
        proto_id: ObjectId,
        name: String,
        description: String,
        start_at: DateTime<Utc>,
        duration_min: u32,
        instructor: ObjectId,
        capacity: u32,
        is_one_time: bool,
    ) -> Training {
        Training {
            id: ObjectId::new(),
            proto_id,
            name,
            description,
            start_at,
            duration_min,
            instructor,
            clients: Vec::new(),
            capacity,
            is_one_time,
            is_canceled: false,
            is_processed: false,
        }
    }

    pub fn with_program(
        program: Program,
        start_at: DateTime<Local>,
        instructor: ObjectId,
        is_one_time: bool,
    ) -> Training {
        Training {
            id: ObjectId::new(),
            proto_id: program.id,
            name: program.name,
            description: program.description,
            start_at: start_at.with_timezone(&Utc),
            duration_min: program.duration_min,
            instructor,
            clients: Vec::new(),
            capacity: program.capacity,
            is_one_time,
            is_canceled: false,
            is_processed: false,
        }
    }

    pub fn with_day_and_training(day: DayId, training: Training) -> Training {
        let start_time = training.get_slot().start_at();
        let start_date = day.local();
        let start_at = start_date
            .with_hour(start_time.hour())
            .unwrap()
            .with_minute(start_time.minute())
            .unwrap();

        Training {
            id: training.id,
            proto_id: training.proto_id,
            name: training.name,
            description: training.description,
            start_at: start_at.with_timezone(&Utc),
            duration_min: training.duration_min,
            instructor: training.instructor,
            clients: vec![],
            capacity: training.capacity,
            is_one_time: training.is_one_time,
            is_canceled: false,
            is_processed: false,
        }
    }

    pub fn get_slot(&self) -> Slot {
        Slot::new(self.start_at, self.duration_min)
    }

    pub fn status(&self, now: DateTime<Local>) -> TrainingStatus {
        if self.is_canceled {
            TrainingStatus::Cancelled
        } else {
            let start_at = self.get_slot().start_at();
            let end_at = start_at + chrono::Duration::minutes(self.duration_min as i64);
            if end_at < now {
                TrainingStatus::Finished
            } else if start_at < now {
                TrainingStatus::InProgress
            } else if start_at - chrono::Duration::minutes(CLOSE_SING_UP as i64) < now {
                TrainingStatus::ClosedToSignup
            } else {
                TrainingStatus::OpenToSignup
            }
        }
    }

    pub fn is_full(&self) -> bool {
        self.clients.len() as u32 >= self.capacity
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

    pub fn day_id(&self) -> DayId {
        DayId::from(self.start_at)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
pub enum TrainingStatus {
    OpenToSignup,
    ClosedToSignup,
    InProgress,
    Cancelled,
    Finished,
}

impl TrainingStatus {
    pub fn can_be_canceled(&self) -> bool {
        matches!(
            self,
            TrainingStatus::OpenToSignup | TrainingStatus::ClosedToSignup
        )
    }

    pub fn can_be_uncanceled(&self) -> bool {
        matches!(self, TrainingStatus::Cancelled)
    }

    pub fn can_sign_out(&self) -> bool {
        matches!(self, TrainingStatus::OpenToSignup)
    }

    pub fn can_sign_in(&self) -> bool {
        matches!(self, TrainingStatus::OpenToSignup)
    }
}
