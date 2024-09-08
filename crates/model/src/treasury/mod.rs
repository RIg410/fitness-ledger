pub mod income;
pub mod outcome;
pub mod subs;
pub mod training;

use crate::{decimal::Decimal, user::User};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use income::Income;
use outcome::Outcome;
use serde::{Deserialize, Serialize};
use subs::SellSubscription;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TreasuryEvent {
    #[serde(rename = "_id")]
    id: ObjectId,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    date_time: DateTime<Utc>,
    event: Event,
    debit: Decimal,
    credit: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Event {
    SellSubscription(SellSubscription),
    Outcome(Outcome),
    Income(Income),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: ObjectId,
    pub tg_id: i64,
    pub name: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub phone: String,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        UserInfo {
            id: user.id,
            tg_id: user.tg_id,
            name: user.name.tg_user_name,
            first_name: user.name.first_name,
            last_name: user.name.last_name,
            phone: user.phone,
        }
    }
}
