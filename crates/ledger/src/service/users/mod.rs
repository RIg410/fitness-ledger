use super::history::History;
use chrono::{DateTime, Local, Utc};
use eyre::{bail, eyre, Result};
use log::info;
use model::{
    rights::{Rights, Rule},
    session::Session,
    statistics::marketing::ComeFrom,
    user::{
        extension::{Birthday, UserExtension},
        sanitize_phone, User, UserName,
    },
};
use mongodb::bson::oid::ObjectId;
use std::{ops::Deref, sync::Arc};
use storage::user::UserStore;
use thiserror::Error;
use tx_macro::tx;

pub mod couch;
pub mod family;
pub mod subscription;

#[derive(Clone)]
pub struct Users {
    pub(super) store: Arc<UserStore>,
    pub(super) logs: History,
}

impl Users {
    pub(crate) fn new(store: Arc<UserStore>, logs: History) -> Self {
        Users { store, logs }
    }

    #[tx]
    pub async fn create(
        &self,
        session: &mut Session,
        tg_id: i64,
        name: UserName,
        phone: String,
        come_from: ComeFrom,
    ) -> Result<ObjectId> {
        let phone = sanitize_phone(&phone);
        let is_first_user = self.store.count(session).await? == 0;
        let rights = if is_first_user {
            Rights::full()
        } else {
            Rights::customer()
        };

        let user = self.get_by_tg_id(session, tg_id).await?;
        if user.is_some() {
            return Err(eyre::eyre!("User {} already exists", tg_id));
        }

        let user = self.get_by_phone(session, &phone).await?;
        if let Some(user) = user {
            self.store.set_tg_id(session, user.id, tg_id).await?;
            self.store.set_name(session, user.id, name).await?;
            Ok(user.id)
        } else {
            let user = User::new(tg_id, name.clone(), rights, phone.clone(), come_from);
            let id = user.id;
            self.store.insert(session, user).await?;
            self.logs.create_user(session, name, phone).await?;
            self.store
                .update_extension(
                    session,
                    UserExtension {
                        id,
                        birthday: None,
                        bought_test_group: false,
                        bought_test_personal: false,
                        bought_first_group: false,
                        bought_first_personal: false,
                    },
                )
                .await?;
            Ok(id)
        }
    }

    pub async fn create_uninit(
        &self,
        session: &mut Session,
        phone: String,
        first_name: String,
        last_name: Option<String>,
        come_from: ComeFrom,
    ) -> Result<User> {
        let phone = sanitize_phone(&phone);

        let user = self.get_by_phone(session, &phone).await?;
        if user.is_some() {
            return Err(eyre::eyre!("User {} already exists", phone));
        }

        let user_name = UserName {
            tg_user_name: None,
            first_name,
            last_name,
        };

        let user = User::new(
            -1,
            user_name.clone(),
            Rights::customer(),
            phone.clone(),
            come_from,
        );
        self.store.insert(session, user.clone()).await?;
        self.logs.create_user(session, user_name, phone).await?;
        self.store
            .update_extension(
                session,
                UserExtension {
                    birthday: None,
                    id: user.id,
                    bought_test_group: false,
                    bought_test_personal: false,
                    bought_first_group: false,
                    bought_first_personal: false,
                },
            )
            .await?;
        Ok(user)
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
        id: ObjectId,
        date: DateTime<Local>,
        forced: bool,
    ) -> Result<(), SetDateError> {
        let mut user = self
            .store
            .get_extension(session, id)
            .await
            .map_err(SetDateError::Common)?;
        if !forced && user.birthday.is_some() {
            return Err(SetDateError::AlreadySet);
        }
        user.birthday = Some(Birthday {
            dt: date.with_timezone(&Utc),
        });
        self.store
            .update_extension(session, user)
            .await
            .map_err(SetDateError::Common)?;
        Ok(())
    }

    #[tx]
    pub async fn edit_user_rule(
        &self,
        session: &mut Session,
        id: ObjectId,
        rule: Rule,
        is_active: bool,
    ) -> Result<()> {
        if is_active {
            self.store.add_rule(session, id, &rule).await?;
            info!("Adding rule {:?} to user {}", rule, id);
        } else {
            self.store.remove_rule(session, id, &rule).await?;
            info!("Removing rule {:?} from user {}", rule, id);
        }

        Ok(())
    }

    #[tx]
    pub async fn set_name(
        &self,
        session: &mut Session,
        id: ObjectId,
        first_name: &str,
        last_name: &str,
    ) -> Result<()> {
        self.store.set_first_name(session, id, first_name).await?;
        self.store.set_last_name(session, id, last_name).await?;
        Ok(())
    }

    #[tx]
    pub async fn set_phone(&self, session: &mut Session, id: ObjectId, phone: &str) -> Result<()> {
        let phone = sanitize_phone(phone);
        self.store.set_phone(session, id, &phone).await?;
        Ok(())
    }
}

impl Users {
    #[tx]
    pub async fn unfreeze(&self, session: &mut Session, id: ObjectId) -> Result<()> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        if user.freeze.is_none() {
            return Ok(());
        }

        self.logs.unfreeze(session, user.id).await?;
        self.store.unfreeze(session, id).await
    }

    #[tx]
    pub async fn freeze(
        &self,
        session: &mut Session,
        id: ObjectId,
        days: u32,
        force: bool,
    ) -> Result<()> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        if !force && user.freeze_days < days {
            bail!("Not enough days.");
        }
        if user.freeze.is_some() {
            bail!("Already frozen");
        }

        self.logs.freeze(session, user.id, days).await?;
        self.store.freeze(session, id, days, force).await?;
        Ok(())
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
