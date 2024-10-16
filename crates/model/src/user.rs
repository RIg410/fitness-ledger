use core::fmt;
use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

use super::rights::Rights;
use crate::{
    couch::CouchInfo,
    statistics::marketing::ComeFrom,
    subscription::{self, Status, UserSubscription},
    training::Training,
};
use chrono::{DateTime, Local, TimeZone as _, Utc};
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
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub couch: Option<CouchInfo>,
    #[serde(default)]
    pub settings: UserSettings,
    #[serde(default)]
    pub come_from: ComeFrom,
}

fn default_created_at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2024, 09, 13, 12, 20, 0)
        .single()
        .unwrap()
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
            is_active: true,
            version: 0,
            subscriptions: vec![],
            freeze_days: 0,
            freeze: None,
            created_at: Utc::now(),
            couch: None,
            settings: UserSettings::default(),
            come_from: ComeFrom::default(),
        }
    }

    pub fn is_couch(&self) -> bool {
        self.couch.is_some()
    }

    pub fn find_subscription(
        &mut self,
        reason: FindFor,
        training: &Training,
    ) -> Option<&mut UserSubscription> {
        let start_at = training.get_slot().start_at();
        self.subscriptions
            .sort_by(|a, b| match (&a.status, &b.status) {
                (Status::Active { start_date: a }, Status::Active { start_date: b }) => a.cmp(b),
                (Status::Active { .. }, Status::NotActive) => Ordering::Greater,
                (Status::NotActive, Status::Active { .. }) => Ordering::Less,
                (Status::NotActive, Status::NotActive) => Ordering::Equal,
            });
        self.subscriptions
            .iter_mut()
            .filter(|s| match s.tp {
                subscription::SubscriptionType::Group {} => !training.is_personal,
                subscription::SubscriptionType::Personal { couch_filter } => {
                    if training.is_personal {
                        if let Some(couch) = couch_filter {
                            training.instructor == couch
                        } else {
                            true
                        }
                    } else {
                        false
                    }
                }
            })
            .find(|s| match reason {
                FindFor::Lock => {
                    if let Status::Active { start_date } = s.status {
                        let expiration_date = start_date.with_timezone(&Local)
                            + chrono::Duration::days(i64::from(s.days));
                        expiration_date > start_at && s.balance > 0
                    } else {
                        s.balance > 0
                    }
                }
                FindFor::Charge => s.locked_balance > 0,
                FindFor::Unlock => s.locked_balance > 0,
            })
    }

    pub fn group_balance(&self) -> Balance {
        let balance = self
            .subscriptions
            .iter()
            .filter(|s| !s.tp.is_personal())
            .map(|s| s.balance)
            .sum();
        let locked_balance = self
            .subscriptions
            .iter()
            .filter(|s| !s.tp.is_personal())
            .map(|s| s.locked_balance)
            .sum();
        Balance {
            balance,
            locked_balance,
        }
    }

    pub fn personal_balance(&self) -> Balance {
        let balance = self
            .subscriptions
            .iter()
            .filter(|s| s.tp.is_personal())
            .map(|s| s.balance)
            .sum();
        let locked_balance = self
            .subscriptions
            .iter()
            .filter(|s| s.tp.is_personal())
            .map(|s| s.locked_balance)
            .sum();
        Balance {
            balance,
            locked_balance,
        }
    }

    pub fn available_balance_for_training(&self, training: &Training) -> u32 {
        self.subscriptions
            .iter()
            .filter(|s| match s.tp {
                subscription::SubscriptionType::Group {} => !training.is_personal,
                subscription::SubscriptionType::Personal { couch_filter } => {
                    if training.is_personal {
                        if let Some(couch) = couch_filter {
                            training.instructor == couch
                        } else {
                            true
                        }
                    } else {
                        false
                    }
                }
            })
            .map(|s| s.balance)
            .sum()
    }

    pub fn gc(&mut self) {
        self.subscriptions.retain(|s| !s.is_empty());
    }
}

pub enum FindFor {
    Lock,
    Charge,
    Unlock,
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

impl Display for UserName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.first_name)?;
        if let Some(last_name) = &self.last_name {
            write!(f, " {}", last_name)?;
        }
        if let Some(tg_user_name) = &self.tg_user_name {
            write!(f, " (@{})", tg_user_name)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserPreSell {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub subscription: UserSubscription,
    pub phone: String,
}

pub fn sanitize_phone(phone: &str) -> String {
    if phone.starts_with("8") {
        ("7".to_string() + &phone[1..])
            .chars()
            .filter_map(|c| if c.is_ascii_digit() { Some(c) } else { None })
            .collect()
    } else {
        phone
            .chars()
            .filter_map(|c| if c.is_ascii_digit() { Some(c) } else { None })
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSettings {
    pub notification: Notification,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    pub notify_by_day: bool,
    pub notify_by_n_hours: Option<u8>,
}

impl Default for UserSettings {
    fn default() -> Self {
        UserSettings {
            notification: Notification {
                notify_by_day: true,
                notify_by_n_hours: Some(1),
            },
        }
    }
}

pub struct Balance {
    pub balance: u32,
    pub locked_balance: u32,
}

impl Balance {
    pub fn is_empty(&self) -> bool {
        self.balance == 0 && self.locked_balance == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::user::sanitize_phone;

    #[test]
    fn test_sanitize_phone_with_special_characters() {
        let phone = "+1 (234) 567-8900";
        let sanitized = sanitize_phone(phone);
        assert_eq!(sanitized, "12345678900");
    }

    #[test]
    fn test_sanitize_phone_with_spaces() {
        let phone = "123 456 7890";
        let sanitized = sanitize_phone(phone);
        assert_eq!(sanitized, "1234567890");
    }

    #[test]
    fn test_sanitize_phone_with_dashes() {
        let phone = "123-456-7890";
        let sanitized = sanitize_phone(phone);
        assert_eq!(sanitized, "1234567890");
    }

    #[test]
    fn test_sanitize_phone_with_dots() {
        let phone = "123.456.7890";
        let sanitized = sanitize_phone(phone);
        assert_eq!(sanitized, "1234567890");
    }

    #[test]
    fn test_sanitize_phone_with_letters() {
        let phone = "123-abc-7890";
        let sanitized = sanitize_phone(phone);
        assert_eq!(sanitized, "1237890");
    }

    #[test]
    fn test_sanitize_phone_with_empty_string() {
        let phone = "";
        let sanitized = sanitize_phone(phone);
        assert_eq!(sanitized, "");
    }

    #[test]
    fn test_sanitize_phone_with_only_special_characters() {
        let phone = "+-()";
        let sanitized = sanitize_phone(phone);
        assert_eq!(sanitized, "");
    }
}
