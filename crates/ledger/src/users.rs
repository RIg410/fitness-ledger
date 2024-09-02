use crate::Ledger;
use chrono::{DateTime, Local};
use eyre::Result;
use log::{info, warn};
use mongodb::bson::oid::ObjectId;
use storage::user::{
    rights::{Rights, Rule},
    User, UserName,
};

impl Ledger {
    pub async fn get_user_by_tg_id(&self, tg_id: i64) -> Result<Option<User>> {
        self.users.get_by_tg_id(tg_id).await
    }

    pub async fn create_user(&self, tg_id: i64, name: UserName, phone: String) -> Result<()> {
        let is_first_user = self.users.count().await? == 0;
        let rights = if is_first_user {
            Rights::full()
        } else {
            Rights::customer()
        };

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
        };
        info!("Creating user: {:?}", user);
        self.users.insert(user).await?;
        Ok(())
    }

    pub async fn user_count(&self) -> Result<u64> {
        self.users.count().await
    }

    pub async fn find_users(&self, query: &str, limit: u64, offset: u64) -> Result<Vec<User>> {
        let keywords = query.split_whitespace().collect::<Vec<_>>();
        self.users.find(&keywords, limit, offset).await
    }

    pub async fn get_instructors(&self) -> Result<Vec<User>> {
        self.users.get_instructors().await
    }

    pub async fn set_user_birthday(
        &self,
        id: i64,
        date: DateTime<Local>,
    ) -> std::result::Result<(), SetDateError> {
        let user = self
            .users
            .get_by_tg_id(id)
            .await
            .map_err(|err| SetDateError::Common(err))?;
        let user = user.ok_or(SetDateError::UserNotFound)?;
        if user.birthday.is_some() {
            return Err(SetDateError::AlreadySet);
        }
        self.users
            .set_birthday(user.tg_id, date)
            .await
            .map_err(|err| SetDateError::Common(err))?;
        Ok(())
    }

    pub async fn block_user(&self, tg_id: i64, is_active: bool) -> Result<()> {
        info!("Blocking user: {}", tg_id);
        warn!("remove subscription!!!!");
        self.users.block(tg_id, is_active).await
    }

    pub async fn edit_user_rule(&self, tg_id: i64, rule: Rule, is_active: bool) -> Result<()> {
        if is_active {
            self.users.add_rule(tg_id, &rule).await?;
            info!("Adding rule {:?} to user {}", rule, tg_id);
        } else {
            self.users.remove_rule(tg_id, &rule).await?;
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