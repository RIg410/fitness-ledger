use std::sync::Arc;

use ai::Ai;
use chrono::Local;
use env::Env;
use eyre::{eyre, Context as _, Result};
use log::error;
use model::decimal::Decimal;
use model::errors::LedgerError;
use model::session::Session;
use model::training::TrainingStatus;
use model::treasury::subs::UserId;
use model::user::family::FindFor;
use model::user::{sanitize_phone, User};
use mongodb::bson::oid::ObjectId;
use service::backup::Backup;
use service::calendar::Calendar;
use service::history::{self, History};
use service::programs::Programs;
use service::requests::Requests;
use service::rewards::Rewards;
use service::subscriptions::Subscriptions;
use service::treasury::Treasury;
use service::users::Users;
use service::{backup, statistics};
use storage::session::Db;
use storage::Storage;
use thiserror::Error;
use tx_macro::tx;

pub mod service;
pub mod training;

pub struct Ledger {
    pub db: Arc<Db>,
    pub users: Users,
    pub calendar: Calendar,
    pub programs: Programs,
    pub treasury: Treasury,
    pub subscriptions: Subscriptions,
    pub history: History,
    pub rewards: Rewards,
    pub statistics: statistics::Statistics,
    pub backup: backup::Backup,
    pub requests: Requests,
    pub yookassa: yookassa::Yookassa,
}

impl Ledger {
    pub fn new(storage: Storage, env: Env) -> Self {
        let backup = Backup::new(storage.clone());

        let history = history::History::new(storage.history.clone());
        let programs = Programs::new(storage.programs.clone());

        let users = Users::new(storage.users, history.clone());
        let calendar = Calendar::new(storage.calendar, users.clone(), programs.clone());

        let treasury = Treasury::new(storage.treasury, history.clone());
        let subscriptions = Subscriptions::new(
            storage.subscriptions,
            history.clone(),
            programs.clone(),
            users.clone(),
        );
        let rewards = Rewards::new(storage.rewards);
        let requests = Requests::new(storage.requests, users.clone());

        let statistics = statistics::Statistics::new(
            calendar.clone(),
            history.clone(),
            users.clone(),
            requests.clone(),
            Ai::new(env.ai_base_url().to_owned(), env.ai_api_key().to_owned()),
            treasury.clone(),
        );

        Ledger {
            users,
            calendar,
            programs,
            db: storage.db,
            treasury,
            subscriptions,
            history,
            rewards,
            statistics,
            backup,
            requests,
            yookassa: yookassa::Yookassa::new(&env),
        }
    }

    pub async fn get_user(&self, session: &mut Session, id: ObjectId) -> Result<User> {
        let mut user = self
            .users
            .get(session, id)
            .await
            .context("get_user")?
            .ok_or_else(|| eyre!("User not found:{:?}", id))?;
        self.users.resolve_family(session, &mut user).await?;
        Ok(user)
    }

    #[tx]
    pub async fn block_user(
        &self,
        session: &mut Session,
        id: ObjectId,
        is_active: bool,
    ) -> Result<()> {
        let mut user = self
            .users
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        let user_id = user.id;
        self.users.resolve_family(session, &mut user).await?;
        let mut payer = user.payer_mut()?;

        if !is_active {
            let users_training = self
                .calendar
                .find_trainings(
                    session,
                    model::training::Filter::Client(user_id),
                    usize::MAX,
                    0,
                )
                .await?;
            for training in users_training {
                if !training.clients.contains(&user_id) {
                    continue;
                }

                let status = training.status(Local::now());
                if !status.can_sign_out() {
                    continue;
                }

                if training.tp.is_not_free() {
                    let sub = payer
                        .find_subscription(FindFor::Unlock, &training)
                        .ok_or_else(|| eyre!("User not found"))?;
                    sub.unlock_balance();
                    self.calendar
                        .sign_out(session, training.id(), user_id)
                        .await?;
                }
            }
            self.users.update(session, &mut payer).await?;
        }
        self.history.block_user(session, user_id, is_active).await?;
        self.users.block_user(session, user_id, is_active).await?;
        Ok(())
    }

    #[tx]
    pub async fn sell_subscription(
        &self,
        session: &mut Session,
        subscription: ObjectId,
        buyer: ObjectId,
        discount: Option<Decimal>,
    ) -> Result<(), SellSubscriptionError> {
        let buyer = self
            .users
            .get(session, buyer)
            .await?
            .ok_or_else(|| SellSubscriptionError::UserNotFound)?;

        let subscription = self
            .subscriptions
            .get(session, subscription)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.history
            .sell_subscription(session, subscription.clone(), buyer.id, discount)
            .await?;

        self.users
            .add_subscription(session, buyer.id, subscription.clone(), discount)
            .await?;

        self.treasury
            .sell(session, buyer.id, subscription, discount)
            .await?;
        Ok(())
    }

    #[tx]
    pub async fn presell_subscription(
        &self,
        session: &mut Session,
        sub_id: ObjectId,
        phone: String,
        first_name: String,
        last_name: Option<String>,
        come_from: model::statistics::source::Source,
        discount: Option<Decimal>,
    ) -> Result<()> {
        let phone = sanitize_phone(&phone);
        let buyer = if let Some(bayer) = self.users.get_by_phone(session, &phone).await? {
            bayer
        } else {
            self.users
                .create_uninit(session, phone, first_name, last_name, come_from)
                .await?
        };

        let subscription = self
            .subscriptions
            .get(session, sub_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        self.history
            .sell_subscription(session, subscription.clone(), buyer.id, discount)
            .await?;

        self.users
            .add_subscription(session, buyer.id, subscription.clone(), discount)
            .await?;

        self.treasury
            .sell(session, buyer.id, subscription, discount)
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
        Ok(())
    }

    #[tx]
    pub async fn edit_program_duration(
        &self,
        session: &mut Session,
        program_id: ObjectId,
        value: u32,
    ) -> Result<()> {
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
        self.programs
            .edit_description(session, id, value.clone())
            .await?;
        self.calendar
            .edit_program_description(session, id, value)
            .await?;
        Ok(())
    }

    #[tx]
    pub async fn delete_employee(
        &self,
        session: &mut Session,
        id: ObjectId,
    ) -> Result<(), LedgerError> {
        let has_trainings = !self
            .calendar
            .find_trainings(session, model::training::Filter::Instructor(id), 1, 0)
            .await?
            .is_empty();

        let user = self
            .users
            .get(session, id)
            .await?
            .ok_or_else(|| LedgerError::UserNotFound(id))?;

        if let Some(employee) = user.employee {
            if employee.reward != Decimal::zero() {
                return Err(LedgerError::EmployeeHasReward { user_id: id });
            }
        } else {
            return Err(LedgerError::UserNotEmployee { user_id: id });
        }

        if has_trainings {
            return Err(LedgerError::CouchHasTrainings(id));
        } else {
            self.users.delete_employee(session, id).await?;
            Ok(())
        }
    }

    #[tx]
    pub async fn add_recalculation_reward(
        &self,
        session: &mut Session,
        couch_id: ObjectId,
        amount: Decimal,
        comment: String,
    ) -> Result<()> {
        let mut user = self.get_user(session, couch_id).await?;

        let employee_info = user
            .employee
            .as_mut()
            .ok_or_else(|| eyre!("User is not couch"))?;
        let reward = employee_info.recalc_reward(user.id, amount, comment);
        self.rewards.add_reward(session, reward).await?;
        self.users
            .update_employee_reward_and_rates(session, user.id, employee_info.reward, None)
            .await?;
        Ok(())
    }

    #[tx]
    pub async fn pay_reward(
        &self,
        session: &mut Session,
        couch_id: ObjectId,
        amount: Decimal,
    ) -> Result<()> {
        let user = self.get_user(session, couch_id).await?;
        let mut employee_info = user.employee.ok_or_else(|| eyre!("User is not couch"))?;
        employee_info.get_reward(amount)?;
        self.history.pay_reward(session, couch_id, amount).await?;
        self.treasury
            .reward_employee(session, UserId::Id(couch_id), amount, &Local::now())
            .await?;
        self.users
            .update_employee_reward_and_rates(session, couch_id, employee_info.reward, None)
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
    #[error("Training is full")]
    TrainingIsFull,
}

impl From<mongodb::error::Error> for SignUpError {
    fn from(e: mongodb::error::Error) -> Self {
        SignUpError::Common(e.into())
    }
}
