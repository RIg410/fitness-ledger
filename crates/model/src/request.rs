use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::statistics::marketing::ComeFrom;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub phone: String,
    pub comment: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    pub come_from: ComeFrom,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

impl Request {
    pub fn new(
        phone: String,
        comment: String,
        come_from: ComeFrom,
        first_name: Option<String>,
        last_name: Option<String>,
    ) -> Request {
        Request {
            id: ObjectId::new(),
            phone,
            comment,
            created_at: Utc::now(),
            come_from,
            first_name,
            last_name,
        }
    }
}
