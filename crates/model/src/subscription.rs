use std::cmp::Ordering;

use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::decimal::Decimal;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Subscription {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub items: u32,
    pub price: Decimal,
    pub version: u32,
    #[serde(default)]
    pub freeze_days: u32,
    #[serde(default)]
    pub expiration_days: u32,
}

impl Subscription {
    pub fn new(
        name: String,
        items: u32,
        price: Decimal,
        freeze_days: u32,
        expiration_days: u32,
    ) -> Self {
        Subscription {
            id: ObjectId::new(),
            name,
            items,
            price,
            version: 0,
            freeze_days,
            expiration_days,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct UserSubscription {
    pub subscription_id: ObjectId,
    pub name: String,
    pub items: u32,
    #[serde(default = "default_days")]
    pub days: u32,
    #[serde(default)]   
    pub status: Status,
}

impl UserSubscription {
    pub fn is_expired(&self, current_date: DateTime<Utc>) -> bool {
        match self.status {
            Status::Active { start_date } => {
                let expiration_date = start_date + chrono::Duration::days(i64::from(self.days));
                current_date > expiration_date
            }
            Status::NotActive => false,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, Status::Active { .. })
    }

    pub fn activate(&mut self, sign_up_date: DateTime<Utc>) {
        self.status = Status::Active {
            start_date: sign_up_date,
        };
    }
}

impl From<Subscription> for UserSubscription {
    fn from(value: Subscription) -> Self {
        UserSubscription {
            subscription_id: value.id,
            name: value.name,
            items: value.items,
            days: value.expiration_days,
            status: Status::NotActive,
        }
    }
}

fn default_days() -> u32 {
    31
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, Eq, Ord)]
pub enum Status {
    #[default]
    NotActive,
    Active {
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        start_date: DateTime<Utc>,
    },
}

impl PartialEq for Status {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Status::NotActive, Status::NotActive) => true,
            (Status::NotActive, Status::Active { .. }) => false,
            (Status::Active { .. }, Status::NotActive) => false,
            (
                Status::Active {
                    start_date: l_start_date,
                },
                Status::Active {
                    start_date: r_start_date,
                },
            ) => l_start_date == r_start_date,
        }
    }
}

impl PartialOrd for Status {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Status::NotActive, Status::NotActive) => Some(Ordering::Equal),
            (Status::NotActive, Status::Active { .. }) => Some(Ordering::Greater),
            (Status::Active { .. }, Status::NotActive) => Some(Ordering::Less),
            (
                Status::Active {
                    start_date: l_start_date,
                },
                Status::Active {
                    start_date: r_start_date,
                },
            ) => l_start_date.partial_cmp(r_start_date),
        }
    }
}
