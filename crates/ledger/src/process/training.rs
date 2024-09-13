use crate::Ledger;
use eyre::{eyre, Result};
use log::{error, info};
use model::{session::Session, training::{Training, TrainingStatus}};
use tx_macro::tx;

pub struct TriningBg {
    ledger: Ledger,
}

impl TriningBg {
    pub fn new(ledger: Ledger) -> TriningBg {
        TriningBg { ledger }
    }

    pub async fn process(&self, session: &mut Session) -> Result<()> {
        let mut cursor = self.ledger.calendar.days_for_process(session).await?;
        let now = chrono::Local::now();
        while let Some(day) = cursor.next(session).await {
            let day = day?;
            for training in day.training {
                if training.is_processed {
                    continue;
                }

                let result = match training.status(now) {
                    TrainingStatus::OpenToSignup { .. }
                    | TrainingStatus::ClosedToSignup
                    | TrainingStatus::InProgress => continue,
                    TrainingStatus::Finished => self.process_finished(session, training).await,
                    TrainingStatus::Cancelled => {
                        if training.get_slot().start_at() < now {
                            self.process_canceled(session, training).await
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

    #[tx]
    async fn process_finished(
        &self,
        session: &mut Session,
        training: Training,
    ) -> Result<()> {
        info!("Finalize training:{:?}", training);
        for client in training.clients {
            let user = self
                .ledger
                .users
                .get(session, client)
                .await?
                .ok_or_else(|| eyre!("User not found"))?;
            if user.reserved_balance == 0 {
                return Err(eyre!("Not enough reserved balance:{}", user.tg_id));
            }
            self.ledger
                .users
                .charge_reserved_balance(session, user.tg_id, 1)
                .await?;
        }
        self.ledger
            .calendar
            .finalized(session, training.start_at)
            .await?;
        Ok(())
    }

    #[tx]
    async fn process_canceled(
        &self,
        session: &mut Session,
        training: Training,
    ) -> Result<()> {
        info!("Finalize canceled training:{:?}", training);
        for client in training.clients {
            let user = self
                .ledger
                .users
                .get(session, client)
                .await?
                .ok_or_else(|| eyre!("User not found"))?;
            if user.reserved_balance == 0 {
                return Err(eyre!("Not enough reserved balance:{}", user.tg_id));
            }

            self.ledger
                .users
                .unblock_balance(session, user.tg_id, 1)
                .await?;
        }
        self.ledger
            .calendar
            .finalized(session, training.start_at)
            .await?;
        Ok(())
    }
}
