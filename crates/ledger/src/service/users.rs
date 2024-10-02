use chrono::{DateTime, Local, Utc};
use eyre::{bail, eyre, Result};
use log::info;
use model::{
    couch::{CouchInfo, Rate},
    decimal::Decimal,
    rights::{Rights, Rule},
    session::Session,
    user::{sanitize_phone, User, UserName},
};
use mongodb::bson::oid::ObjectId;
use std::ops::Deref;
use storage::{pre_sell::PreSellStore, user::UserStore};
use thiserror::Error;
use tx_macro::tx;

use super::history::History;

#[derive(Clone)]
pub struct Users {
    store: UserStore,
    presell: PreSellStore,
    logs: History,
}

impl Users {
    pub(crate) fn new(store: UserStore, presell: PreSellStore, logs: History) -> Self {
        Users {
            store,
            logs,
            presell,
        }
    }

    #[tx]
    pub async fn expire_subscription(&self, session: &mut Session, id: ObjectId) -> Result<()> {
        let expired = self.store.expire_subscription(session, id).await?;
        for subscription in expired {
            self.logs
                .expire_subscription(session, id, subscription)
                .await?;
        }

        Ok(())
    }

    #[tx]
    pub async fn update_couch_rate(
        &self,
        session: &mut Session,
        id: ObjectId,
        rate: Rate,
    ) -> Result<()> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        let couch = user.couch.ok_or_else(|| eyre!("User is not a couch"))?;
        let couch = CouchInfo {
            description: couch.description,
            rate,
            reward: couch.reward,
        };
        self.store.set_couch(session, user.tg_id, &couch).await?;
        // self.logs.update_couch_rate(session, user.id, rate).await;
        Ok(())
    }

    #[tx]
    pub async fn update_couch_description(
        &self,
        session: &mut Session,
        id: ObjectId,
        description: String,
    ) -> Result<()> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        let couch = user.couch.ok_or_else(|| eyre!("User is not a couch"))?;
        let couch = CouchInfo {
            description: description.clone(),
            rate: couch.rate,
            reward: couch.reward,
        };
        self.store.set_couch(session, user.tg_id, &couch).await?;
        // self.logs
        //     .update_couch_description(session, user.id, description)
        //     .await;
        Ok(())
    }

    #[tx]
    pub async fn create(
        &self,
        session: &mut Session,
        tg_id: i64,
        name: UserName,
        phone: String,
    ) -> Result<()> {
        let phone = sanitize_phone(&phone);
        let is_first_user = self.store.count(session).await? == 0;
        let rights = if is_first_user {
            Rights::full()
        } else {
            Rights::customer()
        };

        let user = self.get(session, tg_id).await?;
        if user.is_some() {
            return Err(eyre::eyre!("User {} already exists", tg_id));
        }

        let subscriptions = if let Some(presell) = self.presell.get(session, &phone).await? {
            self.presell.delete(session, &phone).await?;
            vec![presell.subscription]
        } else {
            vec![]
        };

        let user = User {
            tg_id,
            name: name.clone(),
            rights,
            phone: phone.clone(),
            birthday: None,
            balance: subscriptions.iter().map(|s| s.items).sum(),
            is_active: true,
            id: ObjectId::new(),
            reserved_balance: 0,
            subscriptions,
            freeze_days: 0,
            freeze: None,
            version: 0,
            created_at: Utc::now(),
            initiated: false,
            couch: None,
            settings: Default::default(),
        };
        self.store.insert(session, user).await?;
        self.logs.create_user(session, name, phone).await?;
        Ok(())
    }

    #[tx]
    pub async fn make_user_instructor(
        &self,
        session: &mut Session,
        tg_id: i64,
        description: String,
        rate: Rate,
    ) -> Result<()> {
        let user = self
            .store
            .get(session, tg_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        if user.couch.is_some() {
            bail!("Already instructor");
        }

        let couch = CouchInfo {
            description,
            reward: Decimal::zero(),
            rate,
        };
        self.store.set_couch(session, tg_id, &couch).await?;
        // self.logs.make_user_instructor(session, tg_id, couch).await;
        Ok(())
    }

    pub async fn find(
        &self,
        session: &mut Session,
        query: &str,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<User>> {
        let keywords = query.split_whitespace().collect::<Vec<_>>();
        self.store.find(session, &keywords, offset, limit).await
    }

    #[tx]
    pub async fn set_user_birthday(
        &self,
        session: &mut Session,
        id: i64,
        date: DateTime<Local>,
        forced: bool,
    ) -> Result<(), SetDateError> {
        let user = self
            .store
            .get(session, id)
            .await
            .map_err(SetDateError::Common)?;
        let user = user.ok_or(SetDateError::UserNotFound)?;
        if !forced && user.birthday.is_some() {
            return Err(SetDateError::AlreadySet);
        }
        // self.logs.set_user_birthday(session, id, date).await;
        self.store
            .set_birthday(session, user.tg_id, date)
            .await
            .map_err(SetDateError::Common)?;
        Ok(())
    }

    #[tx]
    pub async fn edit_user_rule(
        &self,
        session: &mut Session,
        tg_id: i64,
        rule: Rule,
        is_active: bool,
    ) -> Result<()> {
        // self.logs
        //     .edit_user_rule(session, tg_id, rule, is_active)
        //     .await;
        if is_active {
            self.store.add_rule(session, tg_id, &rule).await?;
            info!("Adding rule {:?} to user {}", rule, tg_id);
        } else {
            self.store.remove_rule(session, tg_id, &rule).await?;
            info!("Removing rule {:?} from user {}", rule, tg_id);
        }

        Ok(())
    }

    #[tx]
    pub async fn freeze(&self, session: &mut Session, tg_id: i64, days: u32) -> Result<()> {
        let user = self
            .store
            .get(session, tg_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        if user.freeze_days < days {
            bail!("Not enough days.");
        }
        if user.freeze.is_some() {
            bail!("Already frozen");
        }

        self.logs.freeze(session, user.id, days).await?;
        self.store.freeze(session, tg_id, days).await?;
        Ok(())
    }

    #[tx]
    pub async fn change_balance(
        &self,
        session: &mut Session,
        tg_id: i64,
        amount: i32,
    ) -> Result<()> {
        let user = self
            .store
            .get(session, tg_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.logs.change_balance(session, user.id, amount).await?;
        self.store.change_balance(session, tg_id, amount).await
    }

    #[tx]
    pub async fn change_reserved_balance(
        &self,
        session: &mut Session,
        tg_id: i64,
        amount: i32,
    ) -> Result<()> {
        let user = self
            .store
            .get(session, tg_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.logs
            .change_reserved_balance(session, user.id, amount)
            .await?;
        self.store
            .change_reserved_balance(session, tg_id, amount)
            .await
    }

    #[tx]
    pub async fn set_name(
        &self,
        session: &mut Session,
        tg_id: i64,
        first_name: &str,
        last_name: &str,
    ) -> Result<()> {
        // self.logs
        //     .set_user_name(session, tg_id, first_name, last_name)
        //     .await;
        self.store
            .set_first_name(session, tg_id, first_name)
            .await?;
        self.store.set_last_name(session, tg_id, last_name).await?;
        Ok(())
    }

    #[tx]
    pub async fn set_phone(&self, session: &mut Session, tg_id: i64, phone: &str) -> Result<()> {
        let phone = sanitize_phone(phone);
        self.store.set_phone(session, tg_id, &phone).await?;
        // self.logs.set_phone(session, tg_id, phone).await;
        Ok(())
    }
}

impl Users {
    pub async fn unfreeze(&self, session: &mut Session, tg_id: i64) -> Result<()> {
        let user = self
            .store
            .get(session, tg_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.logs.unfreeze(session, user.id).await?;
        self.store.unfreeze(session, tg_id).await
    }
}

impl Deref for Users {
    type Target = UserStore;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

#[derive(Debug, Error)]
pub enum SetDateError {
    #[error("User not found")]
    UserNotFound,
    #[error("Birthday already set")]
    AlreadySet,
    #[error(transparent)]
    Common(eyre::Error),
}

impl From<mongodb::error::Error> for SetDateError {
    fn from(e: mongodb::error::Error) -> Self {
        SetDateError::Common(e.into())
    }
}

impl From<eyre::Error> for SetDateError {
    fn from(e: eyre::Error) -> Self {
        SetDateError::Common(e)
    }
}
