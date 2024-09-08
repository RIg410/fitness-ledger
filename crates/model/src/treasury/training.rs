use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::program::Program;

use super::UserInfo;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Training {
    pub instructor: UserInfo,
    pub clients: Vec<UserInfo>,
    pub program: Program,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub start_at: DateTime<Utc>,
}
