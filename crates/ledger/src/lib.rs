use chrono::Local;
use eyre::{bail, eyre, Context as _, Result};
use log::{error, warn};
use model::decimal::Decimal;
use model::session::Session;
use model::subscription::Subscription;
use model::training::{Training, TrainingStatus};
use model::treasury::subs::UserId;
use model::treasury::Sell;
use model::user::{sanitize_phone, User, UserIdent, UserPreSell};
use mongodb::bson::oid::ObjectId;
use service::backup::Backup;
use service::history::{self, History};
use service::programs::Programs;
use service::rewards::Rewards;
use service::subscriptions::Subscriptions;
use service::treasury::Treasury;
use service::users::Users;
use service::{backup, statistics};
use service::calendar::{Calendar, SignOutError};
use storage::pre_sell::PreSellStore;
use storage::session::Db;
use storage::Storage;
use thiserror::Error;

use tx_macro::tx;

pub mod service;

#[derive(Clone)]
pub struct Ledger {
    pub db: Db,
    pub users: Users,
    pub calendar: Calendar,
    pub programs: Programs,
    pub treasury: Treasury,
    pub subscriptions: Subscriptions,
    pub presell: PreSellStore,
    pub history: History,
    pub rewards: Rewards,
    pub statistics: statistics::Statistics,
    pub backup: backup::Backup,
}

impl Ledger {
    pub fn new(storage: Storage) -> Self {
        let backup = Backup::new(storage.clone());
        let history = history::History::new(storage.history);
        let programs = Programs::new(storage.programs, history.clone());
        let calendar = Calendar::new(
            storage.calendar,
            storage.users.clone(),
            programs.clone(),
            history.clone(),
        );
        let users = Users::new(storage.users, storage.presell.clone(), history.clone());
        let treasury = Treasury::new(storage.treasury, history.clone());
        let subscriptions = Subscriptions::new(storage.subscriptions, history.clone());
        let presell = storage.presell.clone();
        let rewards = Rewards::new(storage.rewards);
        let statistics = statistics::Statistics::new(calendar.clone());
        Ledger {
            users,
            calendar,
            programs,
            db: storage.db,
            treasury,
            subscriptions,
            history,
            presell,
            rewards,
            statistics,
            backup,
        }
    }

    pub async fn get_user<ID: Into<UserIdent>>(
        &self,
        session: &mut Session,
        id: ID,
    ) -> Result<User> {
        let id: UserIdent = id.into();
        match id {
            UserIdent::TgId(tg_id) => self.users.get_by_tg_id(session, tg_id).await,
            UserIdent::Id(id) => self.users.get(session, id).await,
        }
        .context("get_user")?
        .ok_or_else(|| eyre!("User not found:{:?}", id))
    }

    #[tx]
    pub async fn block_user(
        &self,
        session: &mut Session,
        tg_id: i64,
        is_active: bool,
    ) -> Result<()> {
        let user = self
            .users
            .get_by_tg_id(session, tg_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        if !is_active {
            let mut reserved_balance = user.reserved_balance;
            let users_training = self
                .calendar
                .find_trainings(
                    session,
                    model::training::Filter::Client(user.id),
                    usize::MAX,
                    0,
                )
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
        self.history.block_user(session, user.id, is_active).await?;
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

        if user.couch.is_some() {
            return Err(SignUpError::UserIsCouch);
        }

        if user.balance == 0 {
            return Err(SignUpError::NotEnoughBalance);
        }
        self.users
            .reserve_balance(session, user.tg_id, 1, training.start_at)
            .await?;

        self.calendar
            .sign_up(session, training.start_at, client)
            .await?;
        self.history
            .sign_up(
                session,
                user.id,
                training.get_slot().start_at(),
                training.name,
            )
            .await?;
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
        self.history
            .sign_up(
                session,
                user.id,
                training.get_slot().start_at(),
                training.name,
            )
            .await?;
        Ok(())
    }

    #[tx]
    pub async fn sell_subscription(
        &self,
        session: &mut Session,
        subscription: ObjectId,
        buyer: i64,
    ) -> Result<(), SellSubscriptionError> {
        let buyer = self
            .users
            .get_by_tg_id(session, buyer)
            .await?
            .ok_or_else(|| SellSubscriptionError::UserNotFound)?;

        let subscription = self
            .subscriptions
            .get(session, subscription)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.history
            .sell_subscription(session, subscription.clone(), buyer.id)
            .await?;

        self.users
            .add_subscription(session, buyer.tg_id, subscription.clone())
            .await?;

        self.treasury
            .sell(session, buyer.id, Sell::Sub(subscription))
            .await?;
        Ok(())
    }

    #[tx]
    pub async fn presell_subscription(
        &self,
        session: &mut Session,
        subscription: ObjectId,
        phone: String,
    ) -> Result<(), SellSubscriptionError> {
        let phone = sanitize_phone(&phone);
        let bayer = self.users.get_by_phone(session, &phone).await?;
        if bayer.is_some() {
            error!("User with phone {} already exists", phone);
            return Err(SellSubscriptionError::InvalidParams);
        }

        let subscription = self
            .subscriptions
            .get(session, subscription)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.history
            .presell_subscription(session, subscription.clone(), phone.clone())
            .await?;

        if self.presell.get(session, &phone).await?.is_some() {
            return Err(SellSubscriptionError::SubscriptionAlreadySold);
        }

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
            .presell(session, phone, Sell::Sub(subscription))
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
    ) -> Result<(), SellSubscriptionError> {
        let buyer = self
            .users
            .get_by_tg_id(session, buyer)
            .await?
            .ok_or_else(|| SellSubscriptionError::UserNotFound)?;

        self.history
            .sell_free_subscription(session, price, item, buyer.id)
            .await?;
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
            .sell(session, buyer.id, Sell::Free(item, price))
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
    ) -> Result<(), SellSubscriptionError> {
        let phone = sanitize_phone(&phone);
        let bayer = self.users.get_by_phone(session, &phone).await?;
        if bayer.is_some() {
            error!("User with phone {} already exists", phone);
            return Err(SellSubscriptionError::InvalidParams);
        }
        self.history
            .presell_free_subscription(session, price, item, phone.clone())
            .await?;

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
            .presell(session, phone, Sell::Free(item, price))
            .await?;
        Ok(())
    }

    #[tx]
    pub async fn edit_program_capacity(
        &self,
        session: &mut Session,
        program_id: ObjectId,
        value: u32,
    ) -> Result<()> {
        self.programs
            .edit_capacity(session, program_id, value)
            .await?;
        self.calendar
            .edit_capacity(session, program_id, value)
            .await?;
        // self.history
        //     .edit_program_capacity(session, program_id, value)
        //     .await;
        Ok(())
    }

    #[tx]
    pub async fn edit_program_duration(
        &self,
        session: &mut Session,
        program_id: ObjectId,
        value: u32,
    ) -> Result<()> {
        // self.history
        //     .edit_program_duration(session, program_id, value)
        //     .await?;
        self.calendar
            .edit_duration(session, program_id, value)
            .await?;
        self.programs
            .edit_duration(session, program_id, value)
            .await?;
        Ok(())
    }

    #[tx]
    pub async fn edit_program_name(
        &self,
        session: &mut Session,
        id: ObjectId,
        value: String,
    ) -> Result<()> {
        // self.history
        //     .edit_program_name(session, id, value.clone())
        //     .await?;
        self.programs.edit_name(session, id, value.clone()).await?;
        self.calendar.edit_program_name(session, id, value).await?;
        Ok(())
    }

    #[tx]
    pub async fn edit_program_description(
        &self,
        session: &mut Session,
        id: ObjectId,
        value: String,
    ) -> Result<()> {
        // self.history
        //     .edit_program_description(session, id, value.clone())
        //     .await?;
        self.programs
            .edit_description(session, id, value.clone())
            .await?;
        self.calendar
            .edit_program_description(session, id, value)
            .await?;
        Ok(())
    }

    #[tx]
    pub async fn delete_couch(&self, session: &mut Session, id: ObjectId) -> Result<bool> {
        let has_trainings = !self
            .calendar
            .find_trainings(session, model::training::Filter::Instructor(id), 1, 0)
            .await?
            .is_empty();
        if has_trainings {
            warn!("Couch has trainings");
            return Ok(false);
        } else {
            // self.history.delete_couch(session, id).await;
            self.users.delete_couch(session, id).await?;
            Ok(true)
        }
    }

    #[tx]
    pub async fn pay_reward(
        &self,
        session: &mut Session,
        couch_id: ObjectId,
        amount: Decimal,
    ) -> Result<()> {
        let user = self.get_user(session, couch_id).await?;
        let mut couch_info = user.couch.ok_or_else(|| eyre!("User is not couch"))?;
        couch_info.get_reward(amount)?;
        self.history.pay_reward(session, couch_id, amount).await?;
        self.treasury
            .reward_employee(session, UserId::Id(couch_id), amount, &Local::now())
            .await?;
        self.users
            .update_couch_reward(session, couch_id, couch_info.reward)
            .await?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum SellSubscriptionError {
    #[error("Subscription not found")]
    SubscriptionNotFound,
    #[error("Subscription already sold")]
    SubscriptionAlreadySold,
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
    #[error("User is couch")]
    UserIsCouch,
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
