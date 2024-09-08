use calendar::Calendar;
use eyre::Result;
use log::{error, info};
use model::training::{Training, TrainingStatus};
use mongodb::ClientSession;
use storage::session::Db;
use storage::Storage;

pub mod calendar;
pub mod programs;
pub mod treasury;
mod users;
use programs::Programs;
use treasury::Treasury;
use tx_macro::tx;
pub use users::*;

#[derive(Clone)]
pub struct Ledger {
    pub db: Db,
    pub users: Users,
    pub calendar: Calendar,
    pub programs: Programs,
    pub treasury: Treasury,
}

impl Ledger {
    pub fn new(storage: Storage) -> Self {
        let programs = Programs::new(storage.training);
        let calendar = Calendar::new(storage.calendar, storage.users.clone(), programs.clone());
        let users = Users::new(storage.users, calendar.clone());
        let treasury = Treasury::new(storage.treasury);
        Ledger {
            users,
            calendar,
            programs,
            db: storage.db,
            treasury,
        }
    }

    pub async fn process(&self, session: &mut ClientSession) -> Result<()> {
        let mut cursor = self.calendar.days_for_process(session).await?;
        let now = chrono::Local::now();
        while let Some(day) = cursor.next(session).await {
            let day = day?;
            for training in day.training {
                if training.is_processed {
                    continue;
                }

                let result = match training.status(now) {
                    TrainingStatus::OpenToSignup | TrainingStatus::ClosedToSignup => continue,
                    TrainingStatus::InProgress | TrainingStatus::Finished => {
                        self.process_finished(session, training).await
                    }
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
        session: &mut ClientSession,
        training: Training,
    ) -> Result<()> {
        info!("Finalize training:{:?}", training);

        Ok(())
    }

    #[tx]
    async fn process_canceled(
        &self,
        session: &mut ClientSession,
        training: Training,
    ) -> Result<()> {
        info!("Finalize canceled training:{:?}", training);

        Ok(())
    }
}
