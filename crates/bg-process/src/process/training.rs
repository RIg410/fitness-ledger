use std::sync::Arc;

use crate::{Ledger, Task};
use async_trait::async_trait;
use eyre::{eyre, Error, Result};
use log::{error, info};
use model::{
    session::Session,
    training::{Statistics, Training, TrainingStatus}, user::family::FindFor,
};
use tx_macro::tx;

#[derive(Clone)]
pub struct TriningBg {
    ledger: Arc<Ledger>,
}

#[async_trait]
impl Task for TriningBg {
    const NAME: &'static str = "training";
    const CRON: &'static str = "every 1 hour";

    async fn process(&mut self) -> Result<(), Error> {
        let mut session = self.ledger.db.start_session().await?;

        let mut cursor = self.ledger.calendar.days_to_process(&mut session).await?;
        let now = chrono::Local::now();
        while let Some(day) = cursor.next(&mut session).await {
            let day = day?;
            for training in day.training {
                if training.is_processed {
                    continue;
                }

                let result = match training.status(now) {
                    TrainingStatus::OpenToSignup { .. }
                    | TrainingStatus::ClosedToSignup
                    | TrainingStatus::InProgress => continue,
                    TrainingStatus::Finished => self.process_finished(&mut session, training).await,
                    TrainingStatus::Cancelled => {
                        if training.get_slot().start_at() < now {
                            self.process_canceled(&mut session, training).await
                        } else {
                            continue;
                        }
                    }
                };
                if let Err(err) = result {
                    error!("Failed to finalize: training:{:#}. Training", err);
                }
            }
        }
        Ok(())
    }
}

impl TriningBg {
    pub fn new(ledger: Arc<Ledger>) -> TriningBg {
        TriningBg { ledger }
    }

    #[tx]
    async fn process_finished(&self, session: &mut Session, training: Training) -> Result<()> {
        info!("Finalize training:{:?}", training);

        let mut statistic = Statistics::default();

        if training.tp.is_not_free() {
            for client in &training.clients {
                let mut user = self
                    .ledger
                    .get_user(session, *client)
                    .await?;
                let mut payer = user.payer_mut()?;
                if let Some(sub) = payer.find_subscription(FindFor::Charge, &training) {
                    if !sub.change_locked_balance() {
                        return Err(eyre!("Not enough balance:{}", user.id));
                    }
                    statistic.earned += sub.item_price();
                } else {
                    return Err(eyre!("Subscription not found for user:{}", user.id));
                }
                self.ledger.users.update(session, &mut user).await?;
            }
        }

        let mut couch = self.ledger.get_user(session, training.instructor).await?;
        if let Some(couch_info) = couch.couch.as_mut() {
            if let Some(reward) = couch_info.collect_training_rewards(&training) {
                statistic.couch_rewards += reward.reward;
                self.ledger.rewards.add_reward(session, reward).await?;
                self.ledger
                    .users
                    .update_couch_reward(session, couch.id, couch_info.reward)
                    .await?;
            }
        }

        self.ledger
            .calendar
            .finalized(session, training.start_at, statistic)
            .await?;
        self.ledger
            .history
            .process_finished(session, &training)
            .await?;
        Ok(())
    }

    #[tx]
    async fn process_canceled(&self, session: &mut Session, training: Training) -> Result<()> {
        info!("Finalize canceled training:{:?}", training);

        self.ledger
            .calendar
            .finalized(session, training.start_at, Statistics::default())
            .await?;
        self.ledger
            .history
            .process_canceled(session, &training)
            .await?;
        Ok(())
    }
}
