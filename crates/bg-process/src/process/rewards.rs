use std::sync::Arc;

use crate::{Ledger, Task};
use async_trait::async_trait;
use chrono::Local;
use eyre::Error;
use model::session::Session;
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
        self.process_rewards(&mut session).await?;
        Ok(())
    }
}

impl RewardsBg {
    pub fn new(ledger: Arc<Ledger>) -> RewardsBg {
        RewardsBg { ledger }
    }

    #[tx]
    async fn process_rewards(&self, session: &mut Session) -> Result<(), Error> {
        let mut users = self
            .ledger
            .users
            .employees_with_ready_fix_reward(&mut *session)
            .await?;

        let now = Local::now();
        for user in users.iter_mut() {
            if let Some(employee) = &mut user.employee {
                if let Some(reward) = employee.collect_fix_rewards(user.id, now)? {
                    self.ledger
                        .rewards
                        .add_reward(&mut *session, reward)
                        .await?;
                    self.ledger
                        .users
                        .update_employee_reward_and_rates(
                            &mut *session,
                            user.id,
                            employee.reward,
                            Some(employee.rates.clone()),
                        )
                        .await?;
                }
            }
        }
        Ok(())
    }
}
