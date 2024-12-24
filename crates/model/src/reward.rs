use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{decimal::Decimal, training::TrainingId};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reward {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "couch")]
    pub employee: ObjectId,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    pub reward: Decimal,
    pub source: RewardSource,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RewardSource {
    #[deprecated]
    Training {
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        start_at: DateTime<Utc>,
        clients: u32,
        name: String,
    },
    TrainingV2 {
        training_id: TrainingId,
        name: String,
        details: Vec<(ObjectId, Decimal)>,
    },
    Fixed {},
    #[deprecated]
    FixedMonthly {},
    Recalc {
        comment: String,
    },
}
