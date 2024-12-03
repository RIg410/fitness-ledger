use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UserExtension {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub birthday: Option<Birthday>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Birthday {
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub dt: DateTime<Utc>,
}
