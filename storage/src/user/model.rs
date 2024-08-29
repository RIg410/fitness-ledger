use super::rights::Rights;
use crate::date_time::{opt_naive_date_deserialize, opt_naive_date_serialize};
use chrono::{DateTime, Local, NaiveDate};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub chat_id: i64,
    pub user_id: String,
    pub name: UserName,
    pub rights: Rights,
    pub phone: String,
    #[serde(serialize_with = "opt_naive_date_serialize")]
    #[serde(deserialize_with = "opt_naive_date_deserialize")]
    pub birthday: Option<NaiveDate>,
    pub reg_date: DateTime<Local>,
    pub balance: u64,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
}

fn default_is_active() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserName {
    pub tg_user_name: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
}
