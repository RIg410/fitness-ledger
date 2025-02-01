use std::{
    cmp::Ordering,
    mem,
    ops::{Deref, DerefMut},
};

use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    subscription::{self, Status, SubscriptionType, UserSubscription},
    training::Training,
};

use super::User;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Family {
    #[serde(default)]
    pub payer_id: Option<ObjectId>,
    #[serde(default)]
    pub is_individual: bool,
    #[serde(skip)]
    pub payer: Option<Box<User>>,
    #[serde(default)]
    pub children_ids: Vec<ObjectId>,
    #[serde(skip)]
    pub children: Vec<User>,
    #[serde(default)]
    pub members: Vec<ObjectId>,
}

impl Family {
    pub fn exists(&self) -> bool {
        self.payer_id.is_some() || !self.children_ids.is_empty()
    }
}

pub struct Payer<U>(U, bool);

impl<U> Payer<U> {
    pub(super) fn new(user: U, owned: bool) -> Self {
        Payer(user, owned)
    }
}

impl Payer<&mut User> {
    pub fn subscriptions_mut(&mut self) -> &mut Vec<UserSubscription> {
        &mut self.0.subscriptions
    }

    pub fn expire(&mut self, now: DateTime<Utc>) -> Vec<UserSubscription> {
        let (expired, actual) = mem::take(&mut self.0.subscriptions).into_iter().fold(
            (Vec::new(), Vec::new()),
            |(mut expired, mut actual), sub| {
                if sub.is_expired(now) {
                    expired.push(sub);
                } else {
                    actual.push(sub);
                }
                (expired, actual)
            },
        );

        self.0.subscriptions = actual;
        expired
    }

    pub fn find_subscription(
        &mut self,
        reason: FindFor,
        training: &Training,
    ) -> Option<&mut UserSubscription> {
        let start_at = training.get_slot().start_at();
        self.0
            .subscriptions
            .sort_by(|a, b| match (&a.status, &b.status) {
                (
                    Status::Active {
                        start_date: _,
                        end_date: a_end_date,
                    },
                    Status::Active {
                        start_date: _,
                        end_date: b_end_date,
                    },
                ) => a_end_date.cmp(b_end_date),
                (Status::Active { .. }, Status::NotActive) => Ordering::Less,
                (Status::NotActive, Status::Active { .. }) => Ordering::Greater,
                (Status::NotActive, Status::NotActive) => Ordering::Equal,
            });
        self.0
            .subscriptions
            .iter_mut()
            .filter(|s| match &s.tp {
                SubscriptionType::Group { program_filter } => {
                    !training.tp.is_personal() && program_filter.contains(&training.proto_id)
                }
                SubscriptionType::Personal { couch_filter } => {
                    if training.tp.is_personal() {
                        training.instructor == *couch_filter
                    } else {
                        false
                    }
                }
            })
            .find(|s| match reason {
                FindFor::Lock => {
                    if let Status::Active {
                        start_date: _,
                        end_date,
                    } = s.status
                    {
                        end_date > start_at && (s.unlimited || s.balance > 0)
                    } else {
                        s.unlimited || s.balance > 0
                    }
                }
                FindFor::Charge => s.unlimited || s.locked_balance > 0,
                FindFor::Unlock => s.unlimited || s.locked_balance > 0,
            })
    }
}

impl<'u> Payer<&User> {
    pub fn is_owner(&self) -> bool {
        self.1
    }

    pub fn subscriptions(&self) -> &[UserSubscription] {
        self.0.subscriptions.as_slice()
    }

    pub fn has_subscription(&self) -> bool {
        !self.0.subscriptions.is_empty()
    }

    pub fn group_balance(&self) -> Balance {
        let mut balance = 0;
        let mut locked_balance = 0;
        let mut unlimited = false;

        for sub in &self.0.subscriptions {
            if !sub.tp.is_personal() {
                balance += sub.balance;
                locked_balance += sub.locked_balance;
                if unlimited {
                    continue;
                }
                unlimited |= sub.unlimited;
            }
        }

        Balance {
            balance,
            locked_balance,
            unlimited,
        }
    }

    pub fn personal_balance(&self) -> Balance {
        let mut balance = 0;
        let mut locked_balance = 0;
        let mut unlimited = false;

        for sub in &self.0.subscriptions {
            if sub.tp.is_personal() {
                balance += sub.balance;
                locked_balance += sub.locked_balance;
                if unlimited {
                    continue;
                }
                unlimited |= sub.unlimited;
            }
        }

        Balance {
            balance,
            locked_balance,
            unlimited,
        }
    }

    pub fn available_balance_for_training(&self, training: &Training) -> u32 {
        self.0
            .subscriptions
            .iter()
            .filter(|s| match &s.tp {
                subscription::SubscriptionType::Group { program_filter } => {
                    !training.tp.is_personal() && program_filter.contains(&training.proto_id)
                }
                subscription::SubscriptionType::Personal { couch_filter } => {
                    if training.tp.is_personal() {
                        training.instructor == *couch_filter
                    } else {
                        false
                    }
                }
            })
            .map(|s| s.balance)
            .sum()
    }
}

impl Deref for Payer<&mut User> {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl AsRef<User> for Payer<&mut User> {
    fn as_ref(&self) -> &User {
        self.0
    }
}

impl AsRef<User> for Payer<&User> {
    fn as_ref(&self) -> &User {
        self.0
    }
}

impl DerefMut for Payer<&mut User> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

pub struct Balance {
    pub balance: u32,
    pub locked_balance: u32,
    pub unlimited: bool,
}

impl Balance {
    pub fn is_empty(&self) -> bool {
        self.balance == 0 && self.locked_balance == 0
    }
}

pub enum FindFor {
    Lock,
    Charge,
    Unlock,
}

#[cfg(test)]
mod tests {
    use bson::oid::ObjectId;
    use chrono::{DateTime, Utc};

    use crate::{
        decimal::Decimal,
        program::TrainingType,
        rights::Rights,
        statistics::marketing::ComeFrom,
        subscription::{Status, SubscriptionType, UserSubscription},
        training::Training,
        user::UserName,
    };

    use super::User;

    fn user(subs: Vec<UserSubscription>) -> User {
        User {
            id: ObjectId::new(),
            tg_id: 0,
            name: UserName {
                tg_user_name: None,
                first_name: "".to_owned(),
                last_name: None,
            },
            rights: Rights::customer(),
            phone: None,
            is_active: true,
            freeze: None,
            subscriptions: subs,
            freeze_days: 1,
            version: 0,
            created_at: Default::default(),
            settings: Default::default(),
            come_from: ComeFrom::default(),
            family: Default::default(),
            employee: Default::default(),
        }
    }

    fn sub(
        items: u32,
        tp: SubscriptionType,
        days: u32,
        start_date: Option<&str>,
    ) -> UserSubscription {
        let status = if let Some(start_date) = start_date {
            let start_date: DateTime<Utc> = start_date.parse().unwrap();
            Status::Active {
                start_date,
                end_date: start_date + chrono::Duration::days(i64::from(days)),
            }
        } else {
            Status::NotActive
        };

        UserSubscription {
            id: ObjectId::new(),
            subscription_id: ObjectId::new(),
            name: "".to_owned(),
            items: 0,
            days,
            status,
            price: Decimal::zero(),
            tp,
            balance: items,
            locked_balance: 0,
            unlimited: false,
            discount: None,
        }
    }

    fn training(start_at: &str, group: bool) -> Training {
        Training::new(
            ObjectId::new(),
            "name".to_owned(),
            "desc".to_owned(),
            start_at.parse::<DateTime<Utc>>().unwrap(),
            1,
            ObjectId::new(),
            1,
            false,
            if group {
                TrainingType::Group { is_free: false }
            } else {
                TrainingType::Personal { is_free: false }
            },
            ObjectId::new(),
        )
    }

    #[test]
    fn test_users_find_subscription() {
        let mut alice = user(vec![]);
        let tr = training("2012-12-12T12:12:12Z", true);
        assert!(alice
            .payer_mut()
            .unwrap()
            .find_subscription(super::FindFor::Lock, &tr)
            .is_none());

        let mut alice = user(vec![sub(
            0,
            SubscriptionType::Group {
                program_filter: vec![tr.proto_id],
            },
            1,
            None,
        )]);
        assert!(alice
            .payer_mut()
            .unwrap()
            .find_subscription(super::FindFor::Lock, &tr)
            .is_none());

        let mut alice = user(vec![sub(
            1,
            SubscriptionType::Group {
                program_filter: vec![tr.proto_id],
            },
            1,
            None,
        )]);
        assert!(alice
            .payer_mut()
            .unwrap()
            .find_subscription(super::FindFor::Lock, &tr)
            .is_some());

        let tr_1 = training("2014-12-12T12:12:12Z", true);
        let mut alice = user(vec![
            sub(
                1,
                SubscriptionType::Group {
                    program_filter: vec![tr.proto_id, tr_1.proto_id],
                },
                1,
                None,
            ),
            sub(
                1,
                SubscriptionType::Group {
                    program_filter: vec![tr.proto_id, tr_1.proto_id],
                },
                30,
                Some("2012-12-11T12:12:12Z"),
            ),
        ]);
        assert!(alice
            .payer_mut()
            .unwrap()
            .find_subscription(super::FindFor::Lock, &tr)
            .unwrap()
            .status
            .is_active());
        assert!(!alice
            .payer_mut()
            .unwrap()
            .find_subscription(super::FindFor::Lock, &tr_1)
            .unwrap()
            .status
            .is_active());
    }
}
