use bson::oid::ObjectId;

use crate::user::{employee::Employee, rate::EmployeeRole};

use super::{day::DayStat, source::Source};
use std::collections::HashMap;

pub struct MonthStatistics {
    pub days: Vec<DayStat>,
    pub marketing: MarketingStat,
    pub subscriptions: Vec<SubscriptionStat>,
    pub treasury: TreasuryIO,
}

impl Default for MonthStatistics {
    fn default() -> Self {
        MonthStatistics {
            days: vec![],
            marketing: MarketingStat {
                source: HashMap::new(),
            },
            subscriptions: vec![],
            treasury: TreasuryIO {
                income_other: 0,
                rent: 0,
                employees: vec![],
                sub_rent: 0,
                other_expense: 0,
                sell_subscriptions: 0,
            },
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

pub struct TreasuryIO {
    pub rent: i64,
    pub sub_rent: i64,
    pub other_expense: i64,
    pub income_other: i64,
    pub employees: Vec<EmployeeStat>,
    pub sell_subscriptions: i64,
}
