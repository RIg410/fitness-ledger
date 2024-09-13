pub mod income;
pub mod outcome;
pub mod subs;
pub mod training;

use crate::{decimal::Decimal, subscription::Subscription, user::User};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use income::Income;
use outcome::Outcome;
use serde::{Deserialize, Serialize};
use subs::{SellSubscription, SubscriptionInfo};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TreasuryEvent {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub date_time: DateTime<Utc>,
    pub user: UserInfo,
    pub event: Event,
    pub debit: Decimal,
    pub credit: Decimal,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Sell {
    Sub(Subscription),
    Free(u32, Decimal),
}
impl Sell {
    pub fn debit(&self) -> Decimal {
        match self {
            Sell::Sub(sub) => sub.price,
            Sell::Free(_, price) => *price,
        }
    }
}

impl From<Sell> for SubscriptionInfo {
    fn from(value: Sell) -> Self {
        match value {
            Sell::Sub(sub) => sub.into(),
            Sell::Free(items, price) => SubscriptionInfo {
                id: ObjectId::new(),
                name: items.to_string(),
                items,
                price,
                version: 0,
                free: true,
            },
        }
    }
}
