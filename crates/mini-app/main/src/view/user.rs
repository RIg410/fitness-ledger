use chrono::{DateTime, Utc};
use eyre::Error;
use model::{
    decimal::Decimal,
    rights::Rights,
    statistics::marketing::ComeFrom,
    subscription::UserSubscription,
    user::{
        employee::Employee,
        extension::Birthday,
        family::Family,
        rate::{EmployeeRole, Interval, Rate},
        Freeze, User, UserName,
    },
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserShortView {
    #[serde(serialize_with = "bson::serde_helpers::serialize_object_id_as_hex_string")]
    pub id: ObjectId,
    pub tg_id: i64,
    pub name: UserName,
    pub phone: Option<String>,
}

impl From<User> for UserShortView {
    fn from(user: User) -> Self {
        UserShortView {
            id: user.id,
            tg_id: user.tg_id,
            name: user.name,
            phone: user.phone,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserView {
    #[serde(serialize_with = "bson::serde_helpers::serialize_object_id_as_hex_string")]
    pub id: ObjectId,
    pub tg_id: i64,
    pub name: UserName,
    pub rights: Rights,
    pub phone: Option<String>,
    pub is_active: bool,
    pub freeze: Option<Freeze>,
    pub subscriptions: Vec<UserSubscriptionView>,
    pub freeze_days: u32,
    pub created_at: DateTime<Utc>,
    pub employee: Option<EmployeeView>,
    pub come_from: ComeFrom,
    pub family: FamilyView,
    pub birthday: Option<Birthday>,
}

impl TryFrom<User> for UserView {
    type Error = Error;

    fn try_from(value: User) -> Result<Self, Self::Error> {
        let subscriptions = value
            .payer()?
            .subscriptions()
            .iter()
            .cloned()
            .map(UserSubscriptionView::from)
            .collect();

        Ok(UserView {
            id: value.id,
            tg_id: value.tg_id,
            name: value.name,
            rights: value.rights,
            phone: value.phone.map(|p| fmt_phone(&p)),
            is_active: value.is_active,
            freeze: value.freeze,
            subscriptions,
            freeze_days: value.freeze_days,
            created_at: value.created_at,
            employee: value.employee.map(EmployeeView::from),
            come_from: value.come_from,
            family: value.family.into(),
            birthday: None,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSubscriptionView {
    #[serde(serialize_with = "bson::serde_helpers::serialize_object_id_as_hex_string")]
    pub id: ObjectId,
    #[serde(serialize_with = "bson::serde_helpers::serialize_object_id_as_hex_string")]
    pub subscription_id: ObjectId,
    pub name: String,
    pub active: Option<SubscriptionActiveView>,
    pub is_group: bool,
    pub balance: u32,
    pub locked_balance: u32,
    pub unlimited: bool,
}

impl From<UserSubscription> for UserSubscriptionView {
    fn from(sub: UserSubscription) -> Self {
        UserSubscriptionView {
            id: sub.id,
            subscription_id: sub.subscription_id,
            name: sub.name,
            active: match sub.status {
                model::subscription::Status::Active {
                    start_date,
                    end_date,
                } => Some(SubscriptionActiveView {
                    start: start_date,
                    end: end_date,
                }),
                model::subscription::Status::NotActive => None,
            },
            is_group: sub.tp.is_group(),
            balance: sub.balance,
            locked_balance: sub.locked_balance,
            unlimited: sub.unlimited,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionActiveView {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FamilyView {
    pub payer: Option<UserShortView>,
    pub children: Vec<UserShortView>,
}

impl From<Family> for FamilyView {
    fn from(family: Family) -> Self {
        FamilyView {
            payer: family.payer.map(|u| UserShortView::from(*u)),
            children: family
                .children
                .into_iter()
                .map(UserShortView::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmployeeView {
    pub role: EmployeeRole,
    pub description: String,
    pub reward: Decimal,
    pub rates: Vec<RateView>,
}

impl From<Employee> for EmployeeView {
    fn from(employee: Employee) -> Self {
        EmployeeView {
            role: employee.role,
            description: employee.description,
            reward: employee.reward,
            rates: employee.rates.into_iter().map(RateView::from).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RateView {
    fix: Option<FixView>,
    group_training: Option<GroupTrainingRate>,
    personal_training: Option<Decimal>,
}

impl From<Rate> for RateView {
    fn from(rate: Rate) -> Self {
        match rate {
            Rate::Fix {
                amount,
                next_payment_date,
                reward_interval: interval,
            } => {
                let fix = FixView {
                    amount,
                    next_payment_date,
                    interval,
                };
                RateView {
                    fix: Some(fix),
                    group_training: None,
                    personal_training: None,
                }
            },
            Rate::GroupTraining {
                percent,
                min_reward,
            } => {
                let training_percent = GroupTrainingRate {
                    percent,
                    min_reward,
                };
                RateView {
                    fix: None,
                    group_training: Some(training_percent),
                    personal_training: None,
                }
            }
            Rate::PersonalTraining { percent } => RateView {
                fix: None,
                group_training: None,
                personal_training: Some(percent),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FixView {
    amount: Decimal,
    next_payment_date: DateTime<Utc>,
    interval: Interval,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FixByTrainingView {
    amount: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupTrainingRate {
    percent: Decimal,
    min_reward: Decimal,
}

pub fn fmt_phone(phone: &str) -> String {
    if phone.len() != 11 {
        return phone.to_string();
    }
    let mut result = String::with_capacity(16);
    result.push_str("+7 (");
    result.push_str(&phone[1..4]);
    result.push_str(") ");
    result.push_str(&phone[4..7]);
    result.push('-');
    result.push_str(&phone[7..9]);
    result.push('-');
    result.push_str(&phone[9..11]);
    result
}
