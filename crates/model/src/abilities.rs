use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Abilities {
    TestGroupSubscription {},
    TestPersonalSubscription {},
    FirstGroupSubscription {
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        #[serde(default)]
        dt: chrono::DateTime<chrono::Utc>,
    },
    FirstPersonalSubscription {
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        #[serde(default)]
        dt: chrono::DateTime<chrono::Utc>,
    },
}

impl Abilities {
    pub fn is_outdated(&self) -> bool {
        match self {
            Abilities::TestGroupSubscription {} => false,
            Abilities::TestPersonalSubscription {} => false,
            Abilities::FirstGroupSubscription { dt } => dt < &chrono::Utc::now(),
            Abilities::FirstPersonalSubscription { dt } => dt < &chrono::Utc::now(),
        }
    }
}
