use std::collections::HashMap;

use bson::oid::ObjectId;

use crate::decimal::Decimal;

#[derive(Default)]
pub struct Statistics {
    pub subscriptions: HashMap<ObjectId, SubscriptionStat>,
    pub training: HashMap<String, TrainingsStat>,
    pub total_freeze: u32,

    pub changed_subscription_days: i64,
    pub changed_subscription_balance: i64,
}

#[derive(Default)]
pub struct TrainingsStat {
    pub count: u64,
    pub cancellations_count: u64,
}

impl TrainingsStat {
    pub fn join(&mut self, other: &Self) {
        self.count += other.count;
        self.cancellations_count += other.cancellations_count;
    }
}

pub struct SubscriptionStat {
    pub name: String,
    pub soult_count: u64,
    pub spent: Decimal,
    pub discount: Decimal,
    pub refunds_sum: Decimal,
    pub expired_sum: Decimal,
    pub expired_trainings: u64,
}

impl SubscriptionStat {
    pub fn new(name: String) -> Self {
        SubscriptionStat {
            name,
            soult_count: 0,
            spent: Decimal::zero(),
            discount: Decimal::zero(),
            refunds_sum: Decimal::zero(),
            expired_sum: Decimal::zero(),
            expired_trainings: 0,
        }
    }

    pub fn join(&mut self, other: &Self) {
        self.soult_count += other.soult_count;
        self.spent += other.spent;
        self.discount += other.discount;
        self.refunds_sum += other.refunds_sum;
        self.expired_sum += other.expired_sum;
        self.expired_trainings += other.expired_trainings;
    }
}
