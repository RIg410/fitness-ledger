use std::sync::Arc;

use crate::{Ledger, Task};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use eyre::Error;
use model::{session::Session, user::employee::Employee};
use mongodb::bson::oid::ObjectId;
use tx_macro::tx;

#[derive(Clone)]
pub struct RewardsBg {
    ledger: Arc<Ledger>,
}

#[async_trait]
impl Task for RewardsBg {
    const NAME: &'static str = "rewards";
    const CRON: &'static str = "every 5 hour";

    async fn process(&mut self) -> Result<(), Error> {
        let mut session = self.ledger.db.start_session().await?;

        let mut users = self.ledger.users.employees(&mut session).await?;

        let now = Local::now();
        for user in users.iter_mut() {
            if let Some(employee) = user.employee.as_mut() {
                let reward = employee.collect_fix_rewards(user.id, now)?;
            }
        }
        Ok(())
    }
}

impl RewardsBg {
    pub fn new(ledger: Arc<Ledger>) -> RewardsBg {
        RewardsBg { ledger }
    }

    #[tx]
    async fn process_rewards(
        &self,
        session: &mut Session,
        couch_id: ObjectId,
        couch: &mut Employee,
        now: DateTime<Local>,
    ) -> Result<(), Error> {
        // if let Some(reward) = couch.collect_monthly_rewards(couch_id, now)? {
        // self.ledger.rewards.add_reward(session, reward).await?;
        // self.ledger
        //     .users
        //     .update_employee_reward(session, couch_id, couch.reward)
        //     .await?;
        // self.ledger
        //     .users
        //     .update_couch_rate_tx_less(session, couch_id, couch.group_rate.clone())
        //     .await?;
        // }
        Ok(())
    }
}
