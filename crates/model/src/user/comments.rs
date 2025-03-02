use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Comment {
    pub text: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    pub author: ObjectId,
    pub id: ObjectId,
}

impl Comment {
    pub fn new(text: String, author: ObjectId) -> Self {
        Self {
            text,
            created_at: Utc::now(),
            author,
            id: ObjectId::new(),
        }
    }
}
