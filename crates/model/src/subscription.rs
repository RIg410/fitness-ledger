use crate::decimal::Decimal;
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Subscription {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub items: u32,
    pub price: Decimal,
    pub version: u32,
    #[serde(default)]
    pub freeze_days: u32,
    #[serde(default)]
    pub expiration_days: u32,
    #[serde(default = "default_user_can_buy")]
    pub user_can_buy: bool,
    #[serde(default)]
    pub subscription_type: SubscriptionType,
}

fn default_user_can_buy() -> bool {
    true
}

impl Subscription {
    pub fn new(
        name: String,
        items: u32,
        price: Decimal,
        freeze_days: u32,
        expiration_days: u32,
        user_can_buy: bool,
        subscription_type: SubscriptionType,
    ) -> Self {
        Subscription {
            id: ObjectId::new(),
            name,
            items,
            price,
            version: 0,
            freeze_days,
            expiration_days,
            user_can_buy,
            subscription_type,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
#[non_exhaustive]
pub struct UserSubscription {
    #[serde(default)]
    pub id: ObjectId,
    pub subscription_id: ObjectId,
    pub name: String,
    items: u32,
    #[serde(default = "default_days")]
    pub days: u32,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    price: Decimal,
    #[serde(default)]
    pub tp: SubscriptionType,
    #[serde(default)]
    pub balance: u32,
    #[serde(default)]
    pub locked_balance: u32,
}

impl UserSubscription {
    pub fn is_expired(&self, current_date: DateTime<Utc>) -> bool {
        match self.status {
            Status::Active { start_date } => {
                let expiration_date = start_date + chrono::Duration::days(i64::from(self.days));
                current_date > expiration_date
            }
            Status::NotActive => false,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, Status::Active { .. })
    }

    pub fn activate(&mut self, sign_up_date: DateTime<Utc>) {
        self.status = Status::Active {
            start_date: sign_up_date,
        };
    }

    pub fn is_empty(&self) -> bool {
        self.balance == 0 && self.locked_balance == 0
    }

    pub fn item_price(&self) -> Decimal {
        if self.items == 0 {
            Decimal::zero()
        } else {
            self.price / Decimal::from(self.items)
        }
    }

    pub fn lock_balance(&mut self, sign_up_date: DateTime<Utc>) -> bool {
        if self.balance == 0 {
            return false;
        }

        if !self.is_active() {
            self.activate(sign_up_date);
        }

        self.balance -= 1;
        self.locked_balance += 1;
        true
    }

    pub fn unlock_balance(&mut self) -> bool {
        if self.locked_balance == 0 {
            return false;
        }

        self.balance += 1;
        self.locked_balance -= 1;
        true
    }

    pub fn change_locked_balance(&mut self) -> bool {
        if self.locked_balance == 0 {
            return false;
        }

        self.locked_balance -= 1;
        true
    }
}

impl From<Subscription> for UserSubscription {
    fn from(value: Subscription) -> Self {
        UserSubscription {
            subscription_id: value.id,
            name: value.name,
            items: value.items,
            days: value.expiration_days,
            status: Status::NotActive,
            price: value.price,
            tp: value.subscription_type,
            balance: value.items,
            locked_balance: 0,
            id: ObjectId::new(),
        }
    }
}

fn default_days() -> u32 {
    31
}

/// Don't reorder variants!
#[derive(Debug, Serialize, Deserialize, Clone, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Status {
    Active {
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        start_date: DateTime<Utc>,
    },
    #[default]
    NotActive,
}

impl Status {
    pub fn is_active(&self) -> bool {
        matches!(self, Status::Active { .. })
    }

    pub fn activate(&mut self, sign_up_date: DateTime<Utc>) {
        *self = Status::Active {
            start_date: sign_up_date,
        };
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash)]
pub enum SubscriptionType {
    Group {},
    Personal { couch_filter: Option<ObjectId> },
}

impl SubscriptionType {
    pub fn is_personal(&self) -> bool {
        matches!(self, SubscriptionType::Personal { .. })
    }
}

impl Default for SubscriptionType {
    fn default() -> Self {
        SubscriptionType::Group {}
    }
}
