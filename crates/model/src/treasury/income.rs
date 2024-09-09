use serde::{Deserialize, Serialize};

use super::UserInfo;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Income {
    pub description: String,
}
