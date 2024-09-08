use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subscription {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub items: u32,
    pub price: u32,
    pub version: u32,
}

