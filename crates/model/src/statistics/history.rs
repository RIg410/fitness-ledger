use std::collections::HashMap;

use bson::oid::ObjectId;
use chrono::{DateTime, Local};
use log::warn;

use crate::{decimal::Decimal, history::HistoryRow, subscription::Subscription};

#[derive(Debug)]
pub struct SubscriptionsStatisticsCollector {
    test_sub_id: ObjectId,
    presell_subs: HashMap<String, Subscription>,
    subs: HashMap<ObjectId, SubscriptionStat>,
    by_users: HashMap<ObjectId, UserStat>,
    from: Option<DateTime<Local>>,
    to: Option<DateTime<Local>>,
}

impl SubscriptionsStatisticsCollector {
    pub fn new(test_sub_id: ObjectId) -> Self {
        Self {
            test_sub_id,
            subs: HashMap::new(),
            presell_subs: HashMap::new(),
            by_users: HashMap::new(),
            from: None,
            to: None,
        }
    }

    pub fn get_unresolved_presells(&self) -> Vec<String> {
        self.presell_subs
            .iter()
            .map(|(phone, _)| phone.to_owned())
            .collect()
    }

    pub fn resolve_presell(&mut self, phone: &str, id: ObjectId) {
        if let Some(sub) = self.presell_subs.remove(phone) {
            self.apply_sub(id, &sub);
        }
    }

    fn apply_sub(&mut self, user_id: ObjectId, subscription: &Subscription) {
        let stat = self
            .subs
            .entry(subscription.id)
            .or_insert_with(|| SubscriptionStat {
                name: subscription.name.clone(),
                total: 0,
                sum: Decimal::zero(),
            });
        stat.total += 1;
        stat.sum += subscription.price;

        let user_stat = self.by_users.entry(user_id).or_default();
        user_stat.total_subs += 1;
        user_stat.sum += subscription.price;
        if subscription.id == self.test_sub_id {
            user_stat.has_test_sub = true;
        }
    }

    pub fn extend(&mut self, history: HistoryRow) {
        let date_time = history.date_time.with_timezone(&Local);
        if let Some(from) = self.from {
            if date_time < from {
                self.from = Some(date_time);
            }
        } else {
            self.from = Some(date_time);
        }

        if let Some(to) = self.to {
            if date_time > to {
                self.to = Some(date_time);
            }
        } else {
            self.to = Some(date_time);
        }

        match history.action {
            crate::history::Action::SellSub { subscription } => {
                let user_id = if let Some(user_id) = history.sub_actors.first() {
                    *user_id
                } else {
                    warn!("No user in history row: {:?}", history.id);
                    return;
                };
                self.apply_sub(user_id, &subscription);
            }
            crate::history::Action::PreSellSub {
                subscription,
                phone,
            } => {
                self.presell_subs.insert(phone, subscription);
            }
            _ => {}
        }
    }

    pub fn finish(self) -> SubscriptionStatistics {
        SubscriptionStatistics {
            subs: self.subs.into_iter().map(|(_, stat)| stat).collect(),
            test_subs_count: self
                .by_users
                .values()
                .filter(|stat| stat.has_test_sub)
                .count() as u32,
            users_buy_test_sub_and_stay: self.by_users.values().fold(0, |acc, stat| {
                if stat.has_test_sub && stat.total_subs > 1 {
                    acc + 1
                } else {
                    acc
                }
            }),
            unresolved_presells: self.presell_subs.len() as u32,
            total_subs_sum: self.by_users.values().map(|stat| stat.sum).sum(),
            subs_count: self.by_users.values().map(|stat| stat.total_subs).sum(),
            from: self.from.unwrap_or_default(),
            to: self.to.unwrap_or_default(),
            people_buys_only_test_sub: self
                .by_users
                .iter()
                .filter_map(|(id, stat)| {
                    if stat.total_subs == 1 && stat.has_test_sub {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct SubscriptionStat {
    pub name: String,
    pub total: u32,
    pub sum: Decimal,
}

#[derive(Default, Debug)]
pub struct UserStat {
    pub has_test_sub: bool,
    pub total_subs: u32,
    pub sum: Decimal,
}

#[derive(Debug)]
pub struct SubscriptionStatistics {
    pub from: DateTime<Local>,
    pub to: DateTime<Local>,
    pub subs: Vec<SubscriptionStat>,
    pub test_subs_count: u32,
    pub subs_count: u32,
    pub total_subs_sum: Decimal,
    pub users_buy_test_sub_and_stay: u32,
    pub unresolved_presells: u32,
    pub people_buys_only_test_sub: Vec<ObjectId>,
}
