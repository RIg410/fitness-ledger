use calendar::Calendar;
use eyre::{eyre, Result};
use log::{error, info};
use model::decimal::Decimal;
use model::training::{Training, TrainingStatus};
use mongodb::bson::oid::ObjectId;
use mongodb::ClientSession;
use storage::session::Db;
use storage::Storage;

pub mod calendar;
pub mod programs;
pub mod subscriptions;
pub mod treasury;
mod users;
use programs::Programs;
use subscriptions::Subscriptions;
use thiserror::Error;
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
    pub subscriptions: Subscriptions,
}

impl Ledger {
    pub fn new(storage: Storage) -> Self {
        let programs = Programs::new(storage.training);
        let calendar = Calendar::new(storage.calendar, storage.users.clone(), programs.clone());
        let users = Users::new(storage.users);
        let treasury = Treasury::new(storage.treasury);
        let subscriptions = Subscriptions::new(storage.subscriptions);
        Ledger {
            users,
            calendar,
            programs,
            db: storage.db,
            treasury,
            subscriptions,
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
    pub async fn sell_subscription(
        &self,
        session: &mut ClientSession,
        subscription: ObjectId,
        buyer: i64,
        seller: i64,
    ) -> Result<(), SellSubscriptionError> {
        let buyer = self
            .users
            .get_by_tg_id(session, buyer)
            .await?
            .ok_or_else(|| SellSubscriptionError::UserNotFound)?;

        let seller = self
            .users
            .get_by_tg_id(session, seller)
            .await?
            .ok_or_else(|| SellSubscriptionError::UserNotFound)?;

        let subscription = self
            .subscriptions
            .get(session, subscription)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.users
            .increment_balance(session, buyer.tg_id, subscription.items)
            .await?;

        self.treasury
            .sell(session, seller, buyer, treasury::Sell::Sub(subscription))
            .await?;
        Ok(())
    }

    #[tx]
    pub async fn sell_free_subscription(
        &self,
        session: &mut ClientSession,
        price: Decimal,
        item: u32,
        buyer: i64,
        seller: i64,
    ) -> Result<(), SellSubscriptionError> {
        let buyer = self
            .users
            .get_by_tg_id(session, buyer)
            .await?
            .ok_or_else(|| SellSubscriptionError::UserNotFound)?;

        let seller = self
            .users
            .get_by_tg_id(session, seller)
            .await?
            .ok_or_else(|| SellSubscriptionError::UserNotFound)?;

        self.users
            .increment_balance(session, buyer.tg_id, item)
            .await?;

        self.treasury
            .sell(session, seller, buyer, treasury::Sell::Free(item, price))
            .await?;
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

#[derive(Error, Debug)]
pub enum SellSubscriptionError {
    #[error("Subscription not found")]
    SubscriptionNotFound,
    #[error("User not found")]
    UserNotFound,
    #[error("invalid params")]
    InvalidParams,
    #[error("{0:?}")]
    Common(#[from] eyre::Error),
}

impl From<mongodb::error::Error> for SellSubscriptionError {
    fn from(value: mongodb::error::Error) -> Self {
        SellSubscriptionError::Common(value.into())
    }
}
