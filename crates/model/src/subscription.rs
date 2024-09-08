use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::decimal::Decimal;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subscription {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub items: u32,
    pub price: Decimal,
    pub version: u32,
}

impl Subscription {
    pub fn new(name: String, items: u32, price: Decimal) -> Self {
        Subscription {
            id: ObjectId::new(),
            name,
            items,
            price,
            version: 0,
        }
    }
}