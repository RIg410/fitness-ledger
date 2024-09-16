use super::rights::Rights;
use crate::subscription::UserSubscription;
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
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub initiated: bool,
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
            balance: 0,
            is_active: true,
            reserved_balance: 0,
            version: 0,
            subscriptions: vec![],
            freeze_days: 0,
            freeze: None,
            created_at: Utc::now(),
            initiated: false,
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
pub struct UserPreSell {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub subscription: UserSubscription,
    pub phone: String,
}

pub fn sanitize_phone(phone: &str) -> String {
    phone
        .chars()
        .filter_map(|c| if c.is_digit(10) { Some(c) } else { None })
        .collect()
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
