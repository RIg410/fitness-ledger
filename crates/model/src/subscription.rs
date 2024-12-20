use crate::{decimal::Decimal, training::Training, user::extension::UserExtension};
use bson::oid::ObjectId;
use chrono::{DateTime, Local, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

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
    #[serde(default)]
    pub requirements: Option<SubRequirements>,
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumIter)]
pub enum SubRequirements {
    TestGroupBuy,
    TestPersonalBuy,
    BuyOnFirstDayGroup,
    BuyOnFirstDayPersonal,
}

impl SubRequirements {
    pub fn into_value(&self) -> u8 {
        match self {
            SubRequirements::TestGroupBuy => 0,
            SubRequirements::TestPersonalBuy => 1,
            SubRequirements::BuyOnFirstDayGroup => 2,
            SubRequirements::BuyOnFirstDayPersonal => 3,
        }
    }

    pub fn from_value(value: u8) -> Option<Self> {
        match value {
            0 => Some(SubRequirements::TestGroupBuy),
            1 => Some(SubRequirements::TestPersonalBuy),
            2 => Some(SubRequirements::BuyOnFirstDayGroup),
            3 => Some(SubRequirements::BuyOnFirstDayPersonal),
            _ => None,
        }
    }
}

fn default_user_can_buy() -> bool {
    false
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
        requirements: Option<SubRequirements>,
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
            requirements,
        }
    }

    pub fn can_user_buy(&self, user_extension: &UserExtension) -> bool {
        if !self.user_can_buy {
            return false;
        }

        if let Some(requirements) = &self.requirements {
            match requirements {
                SubRequirements::TestGroupBuy => {
                    if user_extension.bought_test_group {
                        return false;
                    }
                }
                SubRequirements::TestPersonalBuy => {
                    if user_extension.bought_test_personal {
                        return false;
                    }
                }
                SubRequirements::BuyOnFirstDayGroup => {
                    if user_extension.bought_first_group {
                        return false;
                    }
                }
                SubRequirements::BuyOnFirstDayPersonal => {
                    if user_extension.bought_first_personal {
                        return false;
                    }
                }
            }
        }

        true
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
#[non_exhaustive]
pub struct UserSubscription {
    #[serde(default)]
    pub id: ObjectId,
    pub subscription_id: ObjectId,
    pub name: String,
    pub(crate) items: u32,
    #[serde(default = "default_days")]
    pub days: u32,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub(crate) price: Decimal,
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
            Status::Active {
                start_date: _,
                end_date,
            } => current_date > end_date,
            Status::NotActive => false,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, Status::Active { .. })
    }

    pub fn activate(&mut self, training: &Training) {
        self.status.activate(training, self.days);
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

    pub fn lock_balance(&mut self, training: &Training) -> bool {
        if self.balance == 0 {
            return false;
        }

        if !self.is_active() {
            self.activate(training);
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
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        #[serde(default)]
        end_date: DateTime<Utc>,
    },
    #[default]
    NotActive,
}

impl Status {
    pub fn is_active(&self) -> bool {
        matches!(self, Status::Active { .. })
    }

    pub fn activate(&mut self, training: &Training, expiration_days: u32) {
        let end_date =
            training.get_slot().start_at() + chrono::Duration::days(i64::from(expiration_days));

        let end_date = end_date
            .with_timezone(&Local)
            .date_naive()
            .and_hms_opt(23, 59, 59)
            .and_then(|dt| Local.from_local_datetime(&dt).single())
            .unwrap_or(end_date);
        *self = Status::Active {
            start_date: training.get_slot().start_at().with_timezone(&Utc),
            end_date: end_date.with_timezone(&Utc),
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
