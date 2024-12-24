use crate::{
    decimal::Decimal,
    reward::{Reward, RewardSource},
    training::Training,
};
use bson::oid::ObjectId;
use chrono::{DateTime, Local, Utc};
use eyre::{bail, Error};
use serde::{Deserialize, Serialize};

use super::{
    rate::{EmployeeRole, Rate},
    User,
};

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
        users: &[&User],
    ) -> Option<Reward> {
        if training.clients.is_empty() {
            return None;
        }

        let mut reward = Reward {
            id: ObjectId::new(),
            employee: training.instructor,
            created_at: Local::now().with_timezone(&Utc),
            reward: Decimal::zero(),
            source: RewardSource::TrainingV2 {
                training_id: training.id(),
                name: training.name.clone(),
                details: vec![],
            },
        };
        for rate in &self.rates {
            match rate {
                Rate::FixByTraining { amount } => {
                    reward.reward += *amount;
                }
                _ => {}
            }
        }

        if reward.reward.is_zero() {
            None
        } else {
            self.reward += reward.reward;
            Some(reward)
        }
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
            match rate {
                Rate::Fix {
                    amount,
                    last_payment_date,
                    next_payment_date,
                    interval,
                } => {
                    if date_time < *next_payment_date {
                        continue;
                    }
                    reward.reward += *amount;
                    *last_payment_date = *next_payment_date;
                    *next_payment_date =
                        *next_payment_date + chrono::Duration::from_std(*interval)?;
                }
                _ => {}
            }
        }

        if reward.reward.is_zero() {
            Ok(None)
        } else {
            self.reward += reward.reward;
            Ok(Some(reward))
        }
    }
}
