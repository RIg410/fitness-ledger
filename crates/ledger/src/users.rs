use chrono::{DateTime, Local};
use eyre::Result;
use log::{info, warn};
use model::{
    rights::{Rights, Rule},
    user::{User, UserName},
};
use mongodb::bson::oid::ObjectId;
use storage::user::UserStore;

use crate::calendar::Calendar;

#[derive(Clone)]
pub struct Users {
    store: UserStore,
    calendar: Calendar,
}

impl Users {
    pub(crate) fn new(store: UserStore, calendar: Calendar) -> Self {
        Users { store, calendar }
    }

    pub async fn get_by_tg_id(&self, tg_id: i64) -> Result<Option<User>> {
        self.store.get_by_tg_id(tg_id).await
    }
    pub async fn create(&self, tg_id: i64, name: UserName, phone: String) -> Result<()> {
        let is_first_user = self.store.count().await? == 0;
        let rights = if is_first_user {
            Rights::full()
        } else {
            Rights::customer()
        };

        let user = self.get_by_tg_id(tg_id).await?;
        if user.is_some() {
            return Err(eyre::eyre!("User {} already exists", tg_id));
        }

        let user = User {
            tg_id: tg_id,
            name,
            rights,
            phone,
            birthday: None,
            reg_date: chrono::Local::now(),
            balance: 0,
            is_active: true,
            id: ObjectId::new(),
            reserved_balance: 0,
            version: 0,
        };
        self.store.insert(user).await?;
        Ok(())
    }

    pub async fn count(&self) -> Result<u64> {
        self.store.count().await
    }

    pub async fn find(&self, query: &str, limit: u64, offset: u64) -> Result<Vec<User>> {
        let keywords = query.split_whitespace().collect::<Vec<_>>();
        self.store.find(&keywords, limit, offset).await
    }

    pub async fn instructors(&self) -> Result<Vec<User>> {
        self.store.get_instructors().await
    }

    pub async fn set_user_birthday(
        &self,
        id: i64,
        date: DateTime<Local>,
    ) -> std::result::Result<(), SetDateError> {
        let user = self
            .store
            .get_by_tg_id(id)
            .await
            .map_err(|err| SetDateError::Common(err))?;
        let user = user.ok_or(SetDateError::UserNotFound)?;
        if user.birthday.is_some() {
            return Err(SetDateError::AlreadySet);
        }
        self.store
            .set_birthday(user.tg_id, date)
            .await
            .map_err(|err| SetDateError::Common(err))?;
        Ok(())
    }

    pub async fn block_user(&self, tg_id: i64, is_active: bool) -> Result<()> {
        info!("Blocking user: {}", tg_id);
        warn!("remove subscription!!!!");
        self.store.block(tg_id, is_active).await?;
        Ok(())
    }

    pub async fn edit_user_rule(&self, tg_id: i64, rule: Rule, is_active: bool) -> Result<()> {
        if is_active {
            self.store.add_rule(tg_id, &rule).await?;
            info!("Adding rule {:?} to user {}", rule, tg_id);
        } else {
            self.store.remove_rule(tg_id, &rule).await?;
            info!("Removing rule {:?} from user {}", rule, tg_id);
        }

        Ok(())
    }
}

pub enum SetDateError {
    UserNotFound,
    AlreadySet,
    Common(eyre::Error),
}
