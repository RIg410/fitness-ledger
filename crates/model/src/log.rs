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
}
