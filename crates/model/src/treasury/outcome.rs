use serde::{Deserialize, Serialize};

use super::UserInfo;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Outcome {
    pub buyer: UserInfo,
    pub description: String,
}
