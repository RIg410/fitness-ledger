use std::collections::HashMap;

use bson::oid::ObjectId;

use crate::{decimal::Decimal, user::User};

#[derive(Default)]
pub struct Statistics {
    pub subscriptions: HashMap<ObjectId, Subscription>,
    pub training: HashMap<String, Trainings>,
    pub total_freeze: u64,

    pub changed_subscription_days: i64,
    pub changed_subscription_balance: i64,
}

#[derive(Default)]
pub struct Trainings {
    pub count: u64,
    pub cancellations_count: u64,
}

pub struct Subscription {
    pub name: String,
    pub soult_count: u64,
    pub spent: Decimal,
    pub discount: Decimal,
    pub refunds_sum: Decimal,
    pub expired_sum: Decimal,
    pub expired_trainings: u64,
}
