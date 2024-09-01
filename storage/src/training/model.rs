use chrono::{DateTime, Datelike, Local};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrainingProto {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub description: String,
    pub duration_min: u32,
    pub capacity: u32,
}

impl Default for TrainingProto {
    fn default() -> Self {
        TrainingProto {
            id: ObjectId::new(),
            name: String::new(),
            description: String::new(),
            duration_min: 0,
            capacity: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Training {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub proto_id: ObjectId,
    pub name: String,
    pub description: String,
    pub start_at: DateTime<Local>,
    pub duration_min: u32,
    pub instructor: ObjectId,
    pub clients: Vec<ObjectId>,
    pub capacity: u32,
    pub status: TrainingStatus,
    pub is_one_time: bool,
}

impl Training {
    pub fn start_at(&self) -> DateTime<Local> {
        self.start_at
    }

    pub fn end_at(&self) -> DateTime<Local> {
        self.start_at + chrono::Duration::minutes(self.duration_min as i64)
    }

    pub fn is_full(&self) -> bool {
        self.clients.len() >= self.capacity as usize
    }

    pub fn is_open_to_signup(&self) -> bool {
        self.status == TrainingStatus::OpenToSignup
    }

    pub fn is_training_time(&self, time: DateTime<Local>) -> bool {
        self.start_at <= time && time <= self.end_at()
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
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TrainingStatus {
    OpenToSignup,
    ClosedToSignup,
    InProgress,
    Cancelled,
    Finished,
}
