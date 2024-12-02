use bson::oid::ObjectId;
use chrono::{DateTime, Local, Months, Utc};
use eyre::{bail, eyre, Error};
use serde::{Deserialize, Serialize};

use crate::{decimal::Decimal, training::Training};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CouchInfo {
    pub description: String,
    pub reward: Decimal,
    #[serde(rename = "rate")]
    pub group_rate: GroupRate,
    #[serde(default)]
    pub personal_rate: PersonalRate,
}

impl CouchInfo {
    pub fn recalc_reward(&mut self, id: ObjectId, reward: Decimal, comment: String) -> Reward {
        self.reward += reward;

        Reward {
            id: ObjectId::new(),
            couch: id,
            created_at: Local::now().with_timezone(&Utc),
            reward,
            rate: GroupRate::None,
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

    pub fn collect_training_rewards(&mut self, training: &Training) -> Option<Reward> {
        if training.clients.is_empty() {
            return None;
        }
        match &self.group_rate {
            GroupRate::FixedMonthly { .. } => None,
            GroupRate::PerClient { min, per_client } => {
                let mut reward_sum = *per_client * Decimal::int(training.clients.len() as i64);
                if reward_sum < *min {
                    reward_sum = *min;
                }

                self.reward += reward_sum;
                let reward = Reward {
                    id: ObjectId::new(),
                    couch: training.instructor,
                    created_at: Local::now().with_timezone(&Utc),
                    reward: reward_sum,
                    rate: self.group_rate.clone(),
                    source: RewardSource::Training {
                        start_at: training.get_slot().start_at_utc(),
                        clients: training.clients.len() as u32,
                        name: training.name.clone(),
                    },
                };
                Some(reward)
            }
            GroupRate::None => None,
        }
    }

    pub fn collect_monthly_rewards(
        &mut self,
        id: ObjectId,
        date_time: DateTime<Local>,
    ) -> Result<Option<Reward>, Error> {
        Ok(match self.group_rate.clone() {
            GroupRate::FixedMonthly { rate, next_reward } => {
                let next_reward = next_reward.with_timezone(&Local);
                if date_time > next_reward {
                    self.group_rate = GroupRate::FixedMonthly {
                        rate,
                        next_reward: next_reward
                            .checked_add_months(Months::new(1))
                            .ok_or_else(|| {
                                eyre!("Failed to collect next reward date:{}", next_reward)
                            })?
                            .with_timezone(&Utc),
                    };
                    self.reward += rate;
                    let reward = Reward {
                        id: ObjectId::new(),
                        couch: id,
                        created_at: Local::now().with_timezone(&Utc),
                        reward: rate,
                        rate: self.group_rate.clone(),
                        source: RewardSource::FixedMonthly {},
                    };
                    Some(reward)
                } else {
                    None
                }
            }
            GroupRate::PerClient { .. } => None,
            GroupRate::None => None,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum GroupRate {
    FixedMonthly {
        rate: Decimal,
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        next_reward: DateTime<Utc>,
    },
    PerClient {
        min: Decimal,
        per_client: Decimal,
    },
    #[default]
    None,
}

impl GroupRate {
    pub fn is_fixed_monthly(&self) -> bool {
        matches!(self, GroupRate::FixedMonthly { .. })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalRate {
    // couch_interest is a percentage of the reward that the couch gets
    pub couch_interest: Decimal,
}

impl Default for PersonalRate {
    fn default() -> Self {
        PersonalRate {
            couch_interest: Decimal::int(50),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reward {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub couch: ObjectId,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    pub reward: Decimal,
    pub rate: GroupRate,
    pub source: RewardSource,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RewardSource {
    Training {
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        start_at: DateTime<Utc>,
        clients: u32,
        name: String,
    },
    FixedMonthly {},
    Recalc {
        comment: String,
    },
}
