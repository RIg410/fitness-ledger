use chrono::{DateTime, Local};
use eyre::Error;
use model::{couch::CouchInfo, session::Session};
use mongodb::bson::oid::ObjectId;
use tx_macro::tx;

use crate::Ledger;

pub struct RewardsBg {
    ledger: Ledger,
}

impl RewardsBg {
    pub fn new(ledger: Ledger) -> RewardsBg {
        RewardsBg { ledger }
    }

    pub async fn process(&self, session: &mut Session) -> Result<(), Error> {
        let mut instructors = self.ledger.users.instructors(session).await?;

        let now = Local::now();
        for instructor in instructors.iter_mut() {
            if let Some(couch) = instructor.couch.as_mut() {
                if couch.rate.is_fixed_monthly() {
                    self.process_rewards(session, instructor.id, couch, now)
                        .await?;
                }
            }
        }
        Ok(())
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
