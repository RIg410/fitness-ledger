pub mod aggregate;
pub mod income;
pub mod outcome;
pub mod subs;

use crate::{decimal::Decimal, statistics::source::Source};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use income::Income;
use outcome::Outcome;
use serde::{Deserialize, Serialize};
use subs::{SellSubscription, UserId};

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
    #[serde(default)]
    pub description: Option<String>,
}

impl TreasuryEvent {
    pub fn sum(&self) -> Decimal {
        self.debit - self.credit
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Event {
    // income
    SellSubscription(SellSubscription),
    Income(Income),
    SubRent,
    // outcome
    Rent,
    Outcome(Outcome),
    Reward(UserId),
    Marketing(Source),
}
