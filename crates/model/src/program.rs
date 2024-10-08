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
    pub is_personal: bool,
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
            is_personal: false,
        }
    }
}
