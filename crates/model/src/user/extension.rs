use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UserExtension {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub birthday: Option<Birthday>,
    #[serde(default = "default_buy_flag")]
    pub bought_test_group: bool,
    #[serde(default = "default_buy_flag")]
    pub bought_test_personal: bool,
    #[serde(default = "default_buy_flag")]
    pub bought_first_group: bool,
    #[serde(default = "default_buy_flag")]
    pub bought_first_personal: bool,
}

fn default_buy_flag() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Birthday {
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub dt: DateTime<Utc>,
}
