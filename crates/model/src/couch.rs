use bson::oid::ObjectId;
use chrono::{DateTime, Local, Months, Utc};
use eyre::{bail, eyre, Error};
use serde::{Deserialize, Serialize};

use crate::{decimal::Decimal, training::Training};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Couch {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub description: String,
    pub reward: Decimal,
    pub rate: Rate,
}

impl Couch {
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
    ) -> Result<Option<Reward>, Error> {
        Ok(match &self.rate {
            Rate::FixedMonthly { .. } => None,
            Rate::PerClient { min, per_client } => {
                let mut reward_sum = *per_client * Decimal::int(training.clients.len() as i64);
                if reward_sum < *min {
                    reward_sum = *min;
                }

                self.reward += reward_sum;
                let reward = Reward {
                    id: ObjectId::new(),
                    couch: self.id,
                    created_at: Local::now().with_timezone(&Utc),
                    reward: reward_sum,
                    rate: self.rate.clone(),
                    source: RewardSource::Training {
                        start_at: training.start_at,
                        clients: training.clients.len() as u32,
                    },
                };
                Some(reward)
            }
        })
    }

    pub fn collect_monthly_rewards(
        &mut self,
        date_time: DateTime<Local>,
    ) -> Result<Option<Reward>, Error> {
        Ok(match self.rate.clone() {
            Rate::FixedMonthly { rate, next_reward } => {
                if date_time.with_timezone(&Utc) > next_reward {
                    self.rate = Rate::FixedMonthly {
                        rate,
                        next_reward: next_reward.checked_add_months(Months::new(1)).ok_or_else(
                            || eyre!("Failed to collect next reward date:{}", next_reward),
                        )?,
                    };
                    self.reward += rate;
                    let reward = Reward {
                        id: ObjectId::new(),
                        couch: self.id,
                        created_at: Local::now().with_timezone(&Utc),
                        reward: rate,
                        rate: self.rate.clone(),
                        source: RewardSource::FixedMonthly {},
                    };
                    Some(reward)
                } else {
                    None
                }
            }
            Rate::PerClient { .. } => None,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Rate {
    FixedMonthly {
        rate: Decimal,
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        next_reward: DateTime<Utc>,
    },
    PerClient {
        min: Decimal,
        per_client: Decimal,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reward {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub couch: ObjectId,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    pub reward: Decimal,
    pub rate: Rate,
    pub source: RewardSource,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RewardSource {
    Training {
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        start_at: DateTime<Utc>,
        clients: u32,
    },
    FixedMonthly {},
}
