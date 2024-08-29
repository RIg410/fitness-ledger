use chrono::{DateTime, Local};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Training {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub short_name: String,
    pub description: String,
    pub start_at: DateTime<Local>,
    pub duration_min: u32,
    pub instructor: ObjectId,
    pub clients: Vec<ObjectId>,
    pub capacity: u32,
    pub status: TrainingStatus,
}

impl Training {
    pub fn new(name: &str) -> Training {
        Training {
            short_name: "Растяжка".to_owned(),
            id: ObjectId::new(),
            name: name.to_owned(),
            description: "".to_owned(),
            start_at: Local::now(),
            duration_min: 90,
            instructor: ObjectId::new(),
            clients: vec![],
            capacity: 10,
            status: TrainingStatus::OpenToSignup,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TrainingStatus {
    OpenToSignup,
    ClosedToSignup,
    InProgress,
    Full,
    Cancelled,
    Finished,
}
