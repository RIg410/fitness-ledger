use crate::date_time::{opt_naive_date_deserialize, opt_naive_date_serialize};
use chrono::{DateTime, Local, NaiveDate};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use super::rights::Rights;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserName {
    pub tg_user_name: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
}
