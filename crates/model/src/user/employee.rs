use crate::{
    decimal::Decimal,
    errors::LedgerError,
    reward::{Reward, RewardSource},
    training::Training,
};
use bson::oid::ObjectId;
use chrono::{DateTime, Local, Utc};
use eyre::{bail, Error};
use serde::{Deserialize, Serialize};

use super::rate::{EmployeeRole, Rate};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Employee {
    pub role: EmployeeRole,
    pub description: String,
    pub reward: Decimal,
    pub rates: Vec<Rate>,
}

impl Employee {
    pub fn recalc_reward(&mut self, id: ObjectId, reward: Decimal, comment: String) -> Reward {
        self.reward += reward;

        Reward {
            id: ObjectId::new(),
            employee: id,
            created_at: Local::now().with_timezone(&Utc),
            reward,
            source: RewardSource::Recalc { comment },
        }
    }

    pub fn get_reward(&mut self, take: Decimal) -> Result<(), Error> {
        if take > self.reward {
            bail!(
                "Failed to get rewards. Not enough balance:{} {}",
                take,
                self.reward
            );
        }
        self.reward -= take;
        Ok(())
    }

    pub fn collect_training_rewards(
        &mut self,
        training: &Training,
        users: Vec<UserRewardContribution>,
    ) -> Result<Option<Reward>, LedgerError> {
        if training.clients.is_empty() {
            return Ok(None);
        }

        if training.clients.len() != users.len() {
            return Err(LedgerError::WrongTrainingClients {
                training_id: training.id(),
            });
        }
        for user in &users {
            if !training.clients.contains(&user.user) {
                return Err(LedgerError::WrongTrainingClients {
                    training_id: training.id(),
                });
            }
        }

        let sum = users.iter().map(|u| u.lesson_price).sum::<Decimal>();

        let mut reward = Decimal::zero();
        let mut percent = Decimal::zero();

        for rate in self.rates.as_mut_slice() {
            if training.is_group() {
                if let Rate::GroupTraining {
                    percent: rate_percent,
                    min_reward,
                } = rate
                {
                    reward = sum * *rate_percent;
                    if reward < *min_reward {
                        reward = *min_reward;
                    }
                    percent = *rate_percent;
                    break;
                }
            } else if let Rate::PersonalTraining {
                percent: rate_percent,
            } = rate
            {
                reward = sum * *rate_percent;
                percent = *rate_percent;
                break;
            }
        }

        Ok(if reward.is_zero() {
            None
        } else {
            self.reward += reward;
            Some(Reward {
                id: ObjectId::new(),
                employee: training.instructor,
                created_at: Utc::now(),
                reward,
                source: RewardSource::Training {
                    training_id: training.id(),
                    name: training.name.clone(),
                    user_originals: users,
                    percent,
                },
            })
        })
    }

    pub fn collect_fix_rewards(
        &mut self,
        id: ObjectId,
        date_time: DateTime<Local>,
    ) -> Result<Option<Reward>, Error> {
        let mut reward = Reward {
            id: ObjectId::new(),
            employee: id,
            created_at: date_time.with_timezone(&Utc),
            reward: Decimal::zero(),
            source: RewardSource::Fixed {},
        };

        for rate in self.rates.as_mut_slice() {
            if let Rate::Fix {
                amount,
                next_payment_date,
                reward_interval: interval,
            } = rate
            {
                if date_time < *next_payment_date {
                    continue;
                }
                reward.reward += *amount;
                *next_payment_date = interval.next_date(*next_payment_date);
                break;
            }
        }

        if reward.reward.is_zero() {
            Ok(None)
        } else {
            self.reward += reward.reward;
            Ok(Some(reward))
        }
    }

    pub fn is_couch(&self) -> bool {
        self.role == EmployeeRole::Couch
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserRewardContribution {
    pub user: ObjectId,
    pub lesson_price: Decimal,
    pub subscription_price: Decimal,
    pub lessons_count: u32,
}
