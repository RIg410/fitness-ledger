use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{decimal::Decimal, training::TrainingId, user::employee::UserRewardContribution};

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
    Training {
        training_id: TrainingId,
        name: String,
        percent: Decimal,
        user_originals: Vec<UserRewardContribution>,
    },
    Fixed {},
    Recalc {
        comment: String,
    },
}
