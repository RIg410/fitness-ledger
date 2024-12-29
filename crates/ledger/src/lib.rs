use std::sync::Arc;

use chrono::{Local, Utc};
use env::Env;
use eyre::{eyre, Context as _, Result};
use log::error;
use model::decimal::Decimal;
use model::errors::LedgerError;
use model::request::{RemindLater, Request, RequestHistoryRow};
use model::session::Session;
use model::statistics::marketing::ComeFrom;
use model::training::{Training, TrainingId, TrainingStatus};
use model::treasury::subs::UserId;
use model::user::family::FindFor;
use model::user::{sanitize_phone, User};
use mongodb::bson::oid::ObjectId;
use service::backup::Backup;
use service::calendar::{Calendar, SignOutError};
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
        let requests = Requests::new(storage.requests);

        let statistics = statistics::Statistics::new(
            calendar.clone(),
            history.clone(),
            users.clone(),
            requests.clone(),
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
    pub async fn create_request(
        &self,
        session: &mut Session,
        phone: String,
        come_from: ComeFrom,
        comment: String,
        first_name: Option<String>,
        last_name: Option<String>,
        remember_later: Option<RemindLater>,
    ) -> Result<()> {
        let phone = sanitize_phone(&phone);
        let user = self.users.get_by_phone(session, &phone).await?;
        if let Some(user) = user {
            self.users
                .update_come_from(session, user.id, come_from)
                .await?;
        }
        if let Some(mut request) = self.requests.get_by_phone(session, &phone).await? {
            request.remind_later = remember_later;
            request.history.push(RequestHistoryRow {
                comment: request.comment.clone(),
                date_time: request.created_at,
            });
            request.created_at = Utc::now();
            request.comment = comment;
            request.come_from = come_from;
            self.requests.update(session, &request).await?;
        } else {
            self.requests
                .create(
                    session,
                    Request::new(
                        phone,
                        comment,
                        come_from,
                        first_name,
                        last_name,
                        remember_later,
                    ),
                )
                .await?;
        }
        Ok(())
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
    pub async fn sign_up(
        &self,
        session: &mut Session,
        id: TrainingId,
        client: ObjectId,
        forced: bool,
    ) -> Result<(), SignUpError> {
        let training = self
            .calendar
            .get_training_by_id(session, id)
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

        if training.clients.len() as u32 >= training.capacity {
            return Err(SignUpError::TrainingIsFull);
        }

        let mut user = self
            .users
            .get(session, client)
            .await?
            .ok_or_else(|| SignUpError::UserNotFound)?;
        let user_id = user.id;

        if user.employee.is_some() {
            return Err(SignUpError::UserIsCouch);
        }

        self.users.resolve_family(session, &mut user).await?;
        let mut payer = user.payer_mut()?;

        if training.tp.is_not_free() {
            let subscription = payer
                .find_subscription(FindFor::Lock, &training)
                .ok_or_else(|| SignUpError::NotEnoughBalance)?;

            if !subscription.lock_balance(&training) {
                return Err(SignUpError::NotEnoughBalance);
            }
            self.users.update(session, &mut payer).await?;
        }

        self.calendar
            .sign_up(session, training.id(), client)
            .await?;
        self.history
            .sign_up(
                session,
                user_id,
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
        id: TrainingId,
        client: ObjectId,
        forced: bool,
    ) -> Result<(), SignOutError> {
        let training = self
            .calendar
            .get_training_by_id(session, id)
            .await?
            .ok_or_else(|| SignOutError::TrainingNotFound)?;
        self.sign_out_tx_less(session, &training, client, forced)
            .await
    }

    pub(crate) async fn sign_out_tx_less(
        &self,
        session: &mut Session,
        training: &Training,
        client: ObjectId,
        forced: bool,
    ) -> Result<(), SignOutError> {
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

        let mut user = self
            .users
            .get(session, client)
            .await?
            .ok_or_else(|| SignOutError::UserNotFound)?;
        self.users.resolve_family(session, &mut user).await?;

        let user_id = user.id;
        let mut payer = user.payer_mut()?;

        if training.tp.is_not_free() {
            let sub = payer
                .find_subscription(FindFor::Unlock, training)
                .ok_or_else(|| SignOutError::NotEnoughReservedBalance)?;

            if !sub.unlock_balance() {
                return Err(SignOutError::NotEnoughReservedBalance);
            }
            self.users.update(session, &mut payer).await?;
        }

        self.calendar
            .sign_out(session, training.id(), client)
            .await?;
        self.history
            .sign_out(
                session,
                user_id,
                training.get_slot().start_at(),
                training.name.clone(),
            )
            .await?;
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
        come_from: model::statistics::marketing::ComeFrom,
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
            .update_employee_reward(session, user.id, employee_info.reward)
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
            .update_employee_reward(session, couch_id, employee_info.reward)
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
