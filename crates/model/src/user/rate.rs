use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::decimal::Decimal;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EmployeeRole {
    Couch,
    Manager,
    Admin,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Rate {
    Fix {
        amount: Decimal,
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        last_payment_date: DateTime<Utc>,
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        next_payment_date: DateTime<Utc>,
        interval: Duration,
    },
    FixByTraining {
        amount: Decimal,
    },
    TrainingPercent {
        percent: Decimal,
        min_reward: Option<Decimal>,
    },
}
