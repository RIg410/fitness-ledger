use calendar::{Calendar, SignOutError};
use chrono::Local;
use eyre::{bail, eyre, Result};
use log::error;
use logs::Logs;
use model::decimal::Decimal;
use model::session::Session;
use model::subscription::Subscription;
use model::training::{Training, TrainingStatus};
use model::treasury::Sell;
use model::user::{sanitize_phone, UserPreSell};
use mongodb::bson::oid::ObjectId;
use storage::pre_sell::PreSellStore;
use storage::session::Db;
use storage::Storage;

pub mod calendar;
pub mod logs;
pub mod process;
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
    pub presell: PreSellStore,
    pub logs: Logs,
}

impl Ledger {
    pub fn new(storage: Storage) -> Self {
        let logs = logs::Logs::new(storage.logs);
        let programs = Programs::new(storage.training, logs.clone());
        let calendar = Calendar::new(
            storage.calendar,
            storage.users.clone(),
            programs.clone(),
            logs.clone(),
        );
        let users = Users::new(storage.users, storage.presell.clone(), logs.clone());
        let treasury = Treasury::new(storage.treasury, logs.clone());
        let subscriptions = Subscriptions::new(storage.subscriptions, logs.clone());
        let presell = storage.presell.clone();
        Ledger {
            users,
            calendar,
            programs,
            db: storage.db,
            treasury,
            subscriptions,
            logs,
            presell,
        }
    }

    #[tx]
    pub async fn block_user(
        &self,
        session: &mut Session,
        tg_id: i64,
        is_active: bool,
    ) -> Result<()> {
        if !is_active {
            let user = self
                .users
                .get_by_tg_id(session, tg_id)
                .await?
                .ok_or_else(|| eyre!("User not found"))?;
            let mut reserved_balance = user.reserved_balance;
            let users_training = self
                .calendar
                .get_users_trainings(session, user.id, usize::MAX, 0)
                .await?;
            for training in users_training {
                if !training.clients.contains(&user.id) {
                    continue;
                }

                let status = training.status(Local::now());
                if !status.can_sign_out() {
                    continue;
                }

                if reserved_balance == 0 {
                    bail!("Not enough reserved balance");
                }
                reserved_balance -= 1;

                self.users
                    .unblock_balance(session, user.tg_id, reserved_balance)
                    .await?;
                self.calendar
                    .sign_out(session, training.start_at, user.id)
                    .await?;
            }
        }
        self.logs.block_user(session, tg_id, is_active).await;
        self.users.block_user(session, tg_id, is_active).await?;
        Ok(())
    }

    #[tx]
    pub async fn sign_up(
        &self,
        session: &mut Session,
        training: &Training,
        client: ObjectId,
        forced: bool,
    ) -> Result<(), SignUpError> {
        let training = self
            .calendar
            .get_training_by_start_at(session, training.get_slot().start_at())
            .await?
            .ok_or_else(|| SignUpError::TrainingNotFound)?;
        let status = training.status(Local::now());
        if !forced && !status.can_sign_in() {
            return Err(SignUpError::TrainingNotOpenToSignUp(status));
        }

        if training.is_processed {
            return Err(SignUpError::TrainingNotOpenToSignUp(status));
        }

        if training.clients.contains(&client) {
            return Err(SignUpError::ClientAlreadySignedUp);
        }

        let user = self
            .users
            .get(session, client)
            .await?
            .ok_or_else(|| SignUpError::UserNotFound)?;
        if user.balance == 0 {
            return Err(SignUpError::NotEnoughBalance);
        }
        self.users
            .reserve_balance(session, user.tg_id, 1, training.start_at)
            .await?;

        self.calendar
            .sign_up(session, training.start_at, client)
            .await?;
        self.logs.sign_up(session, training, user.tg_id).await;
        Ok(())
    }

    #[tx]
    pub async fn sign_out(
        &self,
        session: &mut Session,
        training: &Training,
        client: ObjectId,
        forced: bool,
    ) -> Result<(), SignOutError> {
        let training = self
            .calendar
            .get_training_by_start_at(session, training.get_slot().start_at())
            .await?
            .ok_or_else(|| SignOutError::TrainingNotFound)?;
        let status = training.status(Local::now());
        if !forced && !status.can_sign_out() {
            return Err(SignOutError::TrainingNotOpenToSignOut);
        }

        if training.is_processed {
            return Err(SignOutError::TrainingNotOpenToSignOut);
        }

        if !training.clients.contains(&client) {
            return Err(SignOutError::ClientNotSignedUp);
        }

        let user = self
            .users
            .get(session, client)
            .await?
            .ok_or_else(|| SignOutError::UserNotFound)?;
        if user.reserved_balance == 0 {
            return Err(SignOutError::NotEnoughReservedBalance);
        }

        self.users.unblock_balance(session, user.tg_id, 1).await?;

        self.calendar
            .sign_out(session, training.start_at, client)
            .await?;
        self.logs.sign_out(session, training, user.tg_id).await;
        Ok(())
    }

    #[tx]
    pub async fn sell_subscription(
        &self,
        session: &mut Session,
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

        self.logs
            .sell_subscription(session, subscription.clone(), buyer.tg_id, seller.tg_id)
            .await;

        self.users
            .add_subscription(session, buyer.tg_id, subscription.clone())
            .await?;

        self.treasury
            .sell(session, seller, buyer, Sell::Sub(subscription))
            .await?;
        Ok(())
    }

    #[tx]
    pub async fn presell_subscription(
        &self,
        session: &mut Session,
        subscription: ObjectId,
        phone: String,
        seller: i64,
    ) -> Result<(), SellSubscriptionError> {
        let phone = sanitize_phone(&phone);
        let seller = self
            .users
            .get_by_tg_id(session, seller)
            .await?
            .ok_or_else(|| SellSubscriptionError::UserNotFound)?;
        let bayer = self.users.find_by_phone(session, &phone).await?;
        if bayer.is_some() {
            error!("User with phone {} already exists", phone);
            return Err(SellSubscriptionError::InvalidParams);
        }

        let subscription = self
            .subscriptions
            .get(session, subscription)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.logs
            .presell_subscription(session, subscription.clone(), phone.clone(), seller.tg_id)
            .await;

        self.presell
            .add(
                session,
                UserPreSell {
                    id: ObjectId::new(),
                    subscription: subscription.clone().into(),
                    phone: phone.clone(),
                },
            )
            .await?;

        self.treasury
            .presell(session, seller, phone, Sell::Sub(subscription))
            .await?;
        Ok(())
    }

    #[tx]
    pub async fn sell_free_subscription(
        &self,
        session: &mut Session,
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

        self.logs
            .sell_free_subscription(session, price, item, buyer.tg_id, seller.tg_id)
            .await;
        self.users
            .add_subscription(
                session,
                buyer.tg_id,
                Subscription {
                    id: ObjectId::new(),
                    name: item.to_string(),
                    items: item,
                    price,
                    version: 0,
                    freeze_days: 0,
                    expiration_days: 30,
                },
            )
            .await?;

        self.treasury
            .sell(session, seller, buyer, Sell::Free(item, price))
            .await?;
        Ok(())
    }

    #[tx]
    pub async fn presell_free_subscription(
        &self,
        session: &mut Session,
        price: Decimal,
        item: u32,
        phone: String,
        seller: i64,
    ) -> Result<(), SellSubscriptionError> {
        let phone = sanitize_phone(&phone);
        let seller = self
            .users
            .get_by_tg_id(session, seller)
            .await?
            .ok_or_else(|| SellSubscriptionError::UserNotFound)?;
        let bayer = self.users.find_by_phone(session, &phone).await?;
        if bayer.is_some() {
            error!("User with phone {} already exists", phone);
            return Err(SellSubscriptionError::InvalidParams);
        }
        self.logs
            .presell_free_subscription(session, price, item, phone.clone(), seller.tg_id)
            .await;

        self.presell
            .add(
                session,
                UserPreSell {
                    id: ObjectId::new(),
                    subscription: Subscription {
                        id: ObjectId::new(),
                        name: item.to_string(),
                        items: item,
                        price,
                        version: 0,
                        freeze_days: 0,
                        expiration_days: 30,
                    }
                    .into(),
                    phone: phone.clone(),
                },
            )
            .await?;

        self.treasury
            .presell(session, seller, phone, Sell::Free(item, price))
            .await?;
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

#[derive(Debug, Error)]
pub enum SignUpError {
    #[error("Training not found")]
    TrainingNotFound,
    #[error("Training is not open to sign up")]
    TrainingNotOpenToSignUp(TrainingStatus),
    #[error("Client already signed up")]
    ClientAlreadySignedUp,
    #[error("User not found")]
    UserNotFound,
    #[error("Common error:{0}")]
    Common(#[from] eyre::Error),
    #[error("Not enough balance")]
    NotEnoughBalance,
}

impl From<mongodb::error::Error> for SignUpError {
    fn from(e: mongodb::error::Error) -> Self {
        SignUpError::Common(e.into())
    }
}
