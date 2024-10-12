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
