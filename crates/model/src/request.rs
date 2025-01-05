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
    #[serde(rename = "created_at")]
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub modified_at: DateTime<Utc>,
    #[serde(default)]
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    created: DateTime<Utc>,
    pub come_from: ComeFrom,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[serde(default)]
    pub history: Vec<RequestHistoryRow>,
    #[serde(default)]
    pub remind_later: Option<RemindLater>,
}

impl Request {
    pub fn new(
        phone: String,
        comment: String,
        come_from: ComeFrom,
        first_name: Option<String>,
        last_name: Option<String>,
        remind_later: Option<RemindLater>,
    ) -> Request {
        Request {
            id: ObjectId::new(),
            phone,
            comment,
            modified_at: Utc::now(),
            come_from,
            first_name,
            last_name,
            history: vec![],
            remind_later,
            created: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestHistoryRow {
    pub comment: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub date_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemindLater {
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub date_time: DateTime<Utc>,
    pub user_id: ObjectId,
}
