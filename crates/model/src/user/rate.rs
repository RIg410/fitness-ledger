use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::decimal::Decimal;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum EmployeeRole {
    Couch,
    Manager,
    Admin,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Rate {
    Fix {
        amount: Decimal,
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        last_payment_date: DateTime<Utc>,
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        next_payment_date: DateTime<Utc>,
        interval: Duration,
    },
    GroupTraining {
        percent: Decimal,
        min_reward: Option<Decimal>,
    },
    PersonalTraining {
        percent: Decimal,
    },
}

impl Rate {
    pub fn as_u8(&self) -> u8 {
        match self {
            Rate::Fix { .. } => 0,
            Rate::GroupTraining { .. } => 1,
            Rate::PersonalTraining { .. } => 2,
        }
    }
}
