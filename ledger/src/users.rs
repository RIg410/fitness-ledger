use crate::Ledger;
use eyre::Result;
use log::{info, warn};
use mongodb::bson::oid::ObjectId;
use storage::user::{
    rights::{Rights, Rule},
    User, UserName,
};

impl Ledger {
    pub async fn get_user_by_tg_id(&self, tg_id: i64) -> Result<Option<User>> {
        self.storage.get_by_tg_id(tg_id).await
    }

    pub async fn create_user(&self, tg_id: i64, name: UserName, phone: String) -> Result<()> {
        let is_first_user = self.storage.count().await? == 0;
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
        self.storage.insert(user).await?;
        Ok(())
    }

    pub async fn user_count(&self) -> Result<u64> {
        self.storage.count().await
    }

    pub async fn find_users(&self, query: &str, limit: u64, offset: u64) -> Result<Vec<User>> {
        let keywords = query.split_whitespace().collect::<Vec<_>>();
        self.storage.find(&keywords, limit, offset).await
    }

    pub async fn set_user_birthday(
        &self,
        id: i64,
        date: chrono::NaiveDate,
    ) -> std::result::Result<(), SetDateError> {
        let user = self
            .storage
            .get_by_tg_id(id)
            .await
            .map_err(|err| SetDateError::Common(err))?;
        let user = user.ok_or(SetDateError::UserNotFound)?;
        if user.birthday.is_some() {
            return Err(SetDateError::AlreadySet);
        }
        self.storage
            .set_birthday(user.tg_id, date)
            .await
            .map_err(|err| SetDateError::Common(err))?;
        Ok(())
    }


    pub async fn block_user(&self, tg_id: i64, is_active: bool) -> Result<()> {
        info!("Blocking user: {}", tg_id);
        warn!("remove subscription!!!!");
        self.storage.block(tg_id, is_active).await
    }

    pub async fn edit_user_rule(&self, tg_id: i64, rule: Rule, is_active: bool) -> Result<()> {
        if is_active {
            self.storage.add_rule(tg_id, &rule).await?;
            info!("Adding rule {:?} to user {}", rule, tg_id);
        } else {
            self.storage.remove_rule(tg_id, &rule).await?;
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
