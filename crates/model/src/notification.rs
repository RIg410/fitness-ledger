use std::hash::{DefaultHasher, Hash as _, Hasher as _};

use bson::oid::ObjectId;
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};

use crate::training::TrainingId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub to: ObjectId,
    pub message: String,
    pub sent: bool,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub notified_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub user_notification_time: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub deadline: DateTime<Utc>,
    pub send_exact: bool,
    bassness_id: ObjectId,
}

impl Notification {
    pub fn new(
        to: ObjectId,
        message: String,
        notified_at: DateTime<Local>,
        deadline: DateTime<Local>,
        send_exact: bool,
        id: NotificationId,
    ) -> Notification {
        Notification {
            id: ObjectId::new(),
            to,
            message,
            sent: false,
            created_at: Utc::now(),
            notified_at: notified_at.with_timezone(&Utc),
            deadline: deadline.with_timezone(&Utc),
            bassness_id: id.encode(),
            send_exact,
            user_notification_time: notified_at.with_timezone(&Utc),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub enum NotificationId {
    NotifyAboutTomorrowTraining {
        training_id: TrainingId,
        client_id: ObjectId,
    },
    NotifyTomorrowTraining {
        training_id: TrainingId,
        client_id: ObjectId,
    },
    CancelTraining {
        training_id: TrainingId,
        client_id: ObjectId,
    },
    ExpiredSubscription {
        client_id: ObjectId,
        subscription_id: ObjectId,
    },
    LastSubscriptionItem {
        client_id: ObjectId,
        subscription_id: ObjectId,
    },
    RequestNotification {
        request_id: ObjectId,
    },
}

impl NotificationId {
    pub fn encode(&self) -> ObjectId {
        let mut bytes = [0; 12];
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let first_part = hasher.finish();
        bytes[..8].copy_from_slice(&first_part.to_be_bytes());

        let mut hasher = DefaultHasher::new();
        "sec".hash(&mut hasher);
        self.hash(&mut hasher);
        let second_part = hasher.finish();
        bytes[8..].copy_from_slice(&second_part.to_be_bytes()[..4]);

        ObjectId::from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let id = NotificationId::NotifyAboutTomorrowTraining {
            training_id: TrainingId {
                start_at: Local::now().with_timezone(&Utc),
                room: ObjectId::new(),
            },
            client_id: ObjectId::new(),
        };
        let encoded = id.encode();
        assert_eq!(encoded, id.encode());
    }
}
