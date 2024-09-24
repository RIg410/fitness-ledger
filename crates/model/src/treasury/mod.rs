pub mod aggregate;
pub mod income;
pub mod outcome;
pub mod subs;

use crate::{decimal::Decimal, subscription::Subscription};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use income::Income;
use outcome::Outcome;
use serde::{Deserialize, Serialize};
use subs::{SellSubscription, SubscriptionInfo, UserId};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TreasuryEvent {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub date_time: DateTime<Utc>,
    #[serde(default)]
    pub actor: ObjectId,
    pub event: Event,
    pub debit: Decimal,
    pub credit: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Event {
    SellSubscription(SellSubscription),
    Reward(UserId),
    Outcome(Outcome),
    Income(Income),
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
