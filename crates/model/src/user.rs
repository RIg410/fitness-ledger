use crate::subscription::Subscription;

use super::rights::Rights;
use chrono::{DateTime, Local, Utc};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub tg_id: i64,
    pub name: UserName,
    pub rights: Rights,
    pub phone: String,
    pub birthday: Option<DateTime<Local>>,
    pub reg_date: DateTime<Local>,
    pub balance: u32,
    #[serde(default)]
    pub reserved_balance: u32,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
    #[serde(default)]
    pub freeze: Option<Freeze>,
    #[serde(default)]
    pub subscriptions: Vec<UserSubscription>,
    #[serde(default)]
    pub freeze_days: u32,
    #[serde(default)]
    pub version: u64,
}

impl User {
    pub fn new(tg_id: i64) -> User {
        User {
            id: ObjectId::new(),
            tg_id,
            name: UserName {
                tg_user_name: None,
                first_name: "".to_owned(),
                last_name: None,
            },
            rights: Rights::customer(),
            phone: "".to_owned(),
            birthday: None,
            reg_date: Local::now(),
            balance: 0,
            is_active: true,
            reserved_balance: 0,
            version: 0,
            subscriptions: vec![],
            freeze_days: 0,
            freeze: None,
        }
    }
}

fn default_is_active() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Freeze {
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub freeze_start: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub freeze_end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserName {
    pub tg_user_name: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct UserSubscription {
    pub subscription_id: ObjectId,
    pub name: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub start_date: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub end_date: DateTime<Utc>,
    pub items: u32,
}

impl UserSubscription {
    pub fn with_sub(start_date: DateTime<Utc>, sub: Subscription) -> Self {
        UserSubscription {
            subscription_id: sub.id,
            name: sub.name,
            start_date,
            end_date: start_date + chrono::Duration::days(sub.expiration_days as i64),
            items: sub.items,
        }
    }
}
