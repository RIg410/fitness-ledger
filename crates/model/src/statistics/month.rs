use super::{source::Source, training::TrainingsStat};
use crate::user::rate::EmployeeRole;
use bson::oid::ObjectId;
use std::collections::HashMap;

pub struct MonthStatistics {
    pub training: TrainingsStat,
    pub marketing: MarketingStat,
    pub subscriptions: Vec<SubscriptionStat>,
    pub treasury: TreasuryIO,
}

impl Default for MonthStatistics {
    fn default() -> Self {
        MonthStatistics {
            marketing: MarketingStat {
                source: HashMap::new(),
            },
            subscriptions: vec![],
            treasury: TreasuryIO::default(),
            training: TrainingsStat::default(),
        }
    }
}

pub struct EmployeeStat {
    pub id: ObjectId,
    pub role: EmployeeRole,
    pub name: String,
    pub paid: i64,
}

pub struct MarketingStat {
    pub source: HashMap<Source, SourceStat>,
}

pub struct SourceStat {
    pub buy_test: u64,
    pub buy_subscription: u64,
    pub requests_count: u64,
    pub earned: i64,
    pub spent: i64,
}

pub struct SubscriptionStat {
    pub name: String,
    pub count: u64,
    pub earned: i64,
    pub burned_training: u64,
    pub discount: i64,
}

impl SubscriptionStat {
    pub fn new(name: String) -> Self {
        SubscriptionStat {
            name,
            count: 0,
            earned: 0,
            burned_training: 0,
            discount: 0,
        }
    }
}

#[derive(Default)]
pub struct TreasuryIO {
    pub rent: i64,
    pub sub_rent: i64,
    pub other_expense: i64,
    pub income_other: i64,
    pub employees: Vec<EmployeeStat>,
    pub sell_subscriptions: i64,
    pub marketing: HashMap<Source, i64>,
}
