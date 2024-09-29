use crate::{Ledger, Task};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use eyre::Error;
use model::{couch::CouchInfo, session::Session};
use mongodb::bson::oid::ObjectId;
use tx_macro::tx;

#[derive(Clone)]
pub struct RewardsBg {
    ledger: Ledger,
}

#[async_trait]
impl Task for RewardsBg {
    const NAME: &'static str = "rewards";
    const CRON: &'static str = "every 2 hour";

    async fn process(&mut self) -> Result<(), Error> {
        let mut session = self.ledger.db.start_session().await?;

        let mut instructors = self.ledger.users.instructors(&mut session).await?;

        let now = Local::now();
        for instructor in instructors.iter_mut() {
            if let Some(couch) = instructor.couch.as_mut() {
                if couch.rate.is_fixed_monthly() {
                    self.process_rewards(&mut session, instructor.id, couch, now)
                        .await?;
                }
            }
        }
        Ok(())
    }
}

impl RewardsBg {
    pub fn new(ledger: Ledger) -> RewardsBg {
        RewardsBg { ledger }
    }

    #[tx]
    async fn process_rewards(
        &self,
        session: &mut Session,
        couch_id: ObjectId,
        couch: &mut CouchInfo,
        now: DateTime<Local>,
    ) -> Result<(), Error> {
        if let Some(reward) = couch.collect_monthly_rewards(couch_id, now)? {
            self.ledger.rewards.add_reward(session, reward).await?;
            self.ledger
                .users
                .update_couch_reward(session, couch_id, couch.reward)
                .await?;
        }
        Ok(())
    }
}
