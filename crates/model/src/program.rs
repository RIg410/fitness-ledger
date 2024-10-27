use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Program {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub description: String,
    pub duration_min: u32,
    pub capacity: u32,
    #[serde(default)]
    pub version: u64,
    #[serde(default)]
    pub tp: TrainingType,
}

impl Default for Program {
    fn default() -> Self {
        Program {
            id: ObjectId::new(),
            name: String::new(),
            description: String::new(),
            duration_min: 0,
            capacity: 0,
            version: 0,
            tp: TrainingType::Group,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum TrainingType {
    Group,
    Personal,
    Event,
    FreeEvent,
}

impl TrainingType {
    pub fn is_group(&self) -> bool {
        matches!(self, TrainingType::Group)
    }

    pub fn is_personal(&self) -> bool {
        matches!(self, TrainingType::Personal)
    }

    pub fn is_event(&self) -> bool {
        matches!(self, TrainingType::Event)
    }

    pub fn is_free_event(&self) -> bool {
        matches!(self, TrainingType::FreeEvent)
    }
}

impl Default for TrainingType {
    fn default() -> Self {
        TrainingType::Group
    }
}
