use chrono::{DateTime, Local};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrainingProto {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub short_name: String,
    pub description: String,
    pub duration_min: u32,
    pub capacity: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Training {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub proto_id: ObjectId,
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

impl Training {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TrainingStatus {
    OpenToSignup,
    ClosedToSignup,
    InProgress,
    Full,
    Cancelled,
    Finished,
}
