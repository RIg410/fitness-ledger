use std::{
    default,
    fmt::{self, Display, Formatter},
};

use crate::decimal::Decimal;
use chrono::{DateTime, Months, Utc};
use serde::{Deserialize, Serialize};

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
        next_payment_date: DateTime<Utc>,
        #[serde(default)]
        reward_interval: Interval,
    },
    GroupTraining {
        percent: Decimal,
        min_reward: Decimal,
    },
    PersonalTraining {
        percent: Decimal,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Interval {
    Month { num: u32 },
}

impl Display for Interval {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Interval::Month { num } => write!(f, "{} (месяц)", num),
        }
    }
}

impl Default for Interval {
    fn default() -> Self {
        Interval::Month { num: 1 }
    }
}

impl Interval {
    pub fn next_date(&self, date: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            Interval::Month { num } => date.checked_add_months(Months::new(*num)).unwrap(),
        }
    }
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
