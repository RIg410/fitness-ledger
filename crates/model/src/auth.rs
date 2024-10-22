use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use rand::Rng as _;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthKey {
    #[serde(rename = "_id")]
    pub user_id: ObjectId,
    pub key: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
}

impl AuthKey {
    pub fn gen(user_id: ObjectId) -> Self {
        let mut buf = [0u8; 32];
        rand::thread_rng().fill(&mut buf);
        let key = hex::encode(&buf);
        AuthKey {
            user_id,
            key,
            created_at: Utc::now(),
        }
    }
}
