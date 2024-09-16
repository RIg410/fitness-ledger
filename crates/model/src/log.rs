use crate::{
    decimal::Decimal, program::Program, rights::Rule, subscription::Subscription, treasury::Sell,
    user::UserName,
};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LogEntry {
    pub actor: ObjectId,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub date_time: DateTime<Utc>,
    pub action: Action,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    CreateUser {
        tg_id: i64,
        name: UserName,
        phone: String,
    },
    SetUserBirthday {
        tg_id: i64,
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        birthday: DateTime<Utc>,
    },
    EditUserRule {
        tg_id: i64,
        rule: Rule,
        is_active: bool,
    },
    Freeze {
        tg_id: i64,
        days: u32,
    },
    Unfreeze {
        tg_id: i64,
    },
    ChangeBalance {
        tg_id: i64,
        amount: i32,
    },
    SetUserName {
        tg_id: i64,
        first_name: String,
        last_name: String,
    },
    Sell {
        seller: ObjectId,
        buyer: ObjectId,
        sell: Sell,
    },
    Payment {
        user: ObjectId,
        amount: Decimal,
        description: String,
        date_time: DateTime<Utc>,
    },
    Deposit {
        user: ObjectId,
        amount: Decimal,
        description: String,
        date_time: DateTime<Utc>,
    },
    DeleteSub {
        sub: Subscription,
    },
    CreateSub {
        sub: Subscription,
    },
    CreateProgram {
        program: Program,
    },
    FreeSellSub {
        seller: i64,
        buyer: i64,
        price: Decimal,
        item: u32,
    },
    SellSub {
        seller: i64,
        buyer: i64,
        subscription: Subscription,
    },
    SignOut {
        name: String,
        id: ObjectId,
        proto_id: ObjectId,
        start_at: DateTime<Utc>,
        user_id: i64,
    },
    SignUp {
        name: String,
        id: ObjectId,
        proto_id: ObjectId,
        start_at: DateTime<Utc>,
        user_id: i64,
    },
    BlockUser {
        tg_id: i64,
        is_active: bool,
    },
    CancelTraining {
        name: String,
        id: ObjectId,
        proto_id: ObjectId,
        start_at: DateTime<Utc>,
    },
    RestoreTraining {
        name: String,
        id: ObjectId,
        proto_id: ObjectId,
        start_at: DateTime<Utc>,
    },
    DeleteTraining {
        name: String,
        id: ObjectId,
        proto_id: ObjectId,
        start_at: DateTime<Utc>,
        all: bool,
    },
    Schedule {
        name: String,
        id: ObjectId,
        proto_id: ObjectId,
        start_at: DateTime<Utc>,
        #[serde(default)]
        instructor: ObjectId,
    },
    FinalizedTraining {
        name: String,
        id: ObjectId,
        proto_id: ObjectId,
        start_at: DateTime<Utc>,
        clients: Vec<ObjectId>,
        #[serde(default)]
        instructor: ObjectId,
    },
    FinalizedCanceledTraining {
        name: String,
        id: ObjectId,
        proto_id: ObjectId,
        start_at: DateTime<Utc>,
        clients: Vec<ObjectId>,
        #[serde(default)]
        instructor: ObjectId,
    },
    SetPhone {
        tg_id: i64,
        phone: String,
    },
    PreSellSub {
        seller: i64,
        phone: String,
        subscription: Subscription,
    },
    PreFreeSellSub {
        seller: i64,
        phone: String,
        price: Decimal,
        item: u32,
    },
    ChangeReservedBalance {
        tg_id: i64,
        amount: i32,
    },
}
