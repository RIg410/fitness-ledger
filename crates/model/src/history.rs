use bson::oid::ObjectId;
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    decimal::Decimal,
    subscription::{Subscription, UserSubscription},
    user::UserName,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct HistoryRow {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub actor: ObjectId,
    pub sub_actors: Vec<ObjectId>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub date_time: DateTime<Utc>,
    pub action: Action,
}

impl HistoryRow {
    pub fn new(actor: ObjectId, action: Action) -> Self {
        HistoryRow {
            id: ObjectId::new(),
            actor,
            sub_actors: vec![],
            date_time: Local::now().with_timezone(&Utc),
            action,
        }
    }

    pub fn with_sub_actors(actor: ObjectId, sub_actors: Vec<ObjectId>, action: Action) -> Self {
        HistoryRow {
            id: ObjectId::new(),
            actor,
            sub_actors,
            date_time: Local::now().with_timezone(&Utc),
            action,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    BlockUser {
        is_active: bool,
    },
    SignUp {
        start_at: DateTime<Local>,
        name: String,
    },
    SignOut {
        start_at: DateTime<Local>,
        name: String,
    },
    SellSub {
        subscription: Subscription,
        #[serde(default)]
        discount: Option<Decimal>,
    },
    #[deprecated]
    PreSellSub {
        subscription: Subscription,
        phone: String,
    },
    FinalizedCanceledTraining {
        name: String,
        start_at: DateTime<Utc>,
    },
    FinalizedTraining {
        name: String,
        start_at: DateTime<Utc>,
    },
    Payment {
        amount: Decimal,
        description: String,
        date_time: DateTime<Utc>,
    },
    Deposit {
        amount: Decimal,
        description: String,
        date_time: DateTime<Utc>,
    },
    CreateUser {
        name: UserName,
        phone: String,
    },
    Freeze {
        days: u32,
    },
    Unfreeze {},
    ChangeBalance {
        amount: i32,
    },
    ChangeReservedBalance {
        amount: i32,
    },
    PayReward {
        amount: Decimal,
    },
    ExpireSubscription {
        subscription: UserSubscription,
    },
    BuySub {
        subscription: Subscription,
        #[serde(default)]
        discount: Option<Decimal>,
    },
    RemoveFamilyMember {},
    AddFamilyMember {},
}
