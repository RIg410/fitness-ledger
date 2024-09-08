use chrono::{DateTime, Local};
use eyre::{Error, Result};
use log::{info, warn};
use model::{
    rights::{Rights, Rule},
    user::{User, UserName},
};
use mongodb::{bson::oid::ObjectId, ClientSession};
use storage::user::UserStore;
use tx_macro::tx;

#[derive(Clone)]
pub struct Users {
    store: UserStore,
}

impl Users {
    pub(crate) fn new(store: UserStore) -> Self {
        Users { store }
    }

    pub async fn get_by_tg_id(
        &self,
        session: &mut ClientSession,
        tg_id: i64,
    ) -> Result<Option<User>> {
        self.store.get_by_tg_id(session, tg_id).await
    }

    #[tx]
    pub async fn create(
        &self,
        session: &mut ClientSession,
        tg_id: i64,
        name: UserName,
        phone: String,
    ) -> Result<()> {
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
        self.store.insert(session, user).await?;
        Ok(())
    }

    pub async fn count(&self, session: &mut ClientSession) -> Result<u64> {
        self.store.count(session).await
    }

    pub async fn find(
        &self,
        session: &mut ClientSession,
        query: &str,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<User>> {
        let keywords = query.split_whitespace().collect::<Vec<_>>();
        self.store.find(session, &keywords, limit, offset).await
    }

    pub async fn instructors(&self, session: &mut ClientSession) -> Result<Vec<User>> {
        self.store.get_instructors(session).await
    }

    pub async fn set_user_birthday(
        &self,
        session: &mut ClientSession,
        id: i64,
        date: DateTime<Local>,
    ) -> std::result::Result<(), SetDateError> {
        let user = self
            .store
            .get_by_tg_id(session, id)
            .await
            .map_err(|err| SetDateError::Common(err))?;
        let user = user.ok_or(SetDateError::UserNotFound)?;
        if user.birthday.is_some() {
            return Err(SetDateError::AlreadySet);
        }
        self.store
            .set_birthday(session, user.tg_id, date)
            .await
            .map_err(|err| SetDateError::Common(err))?;
        Ok(())
    }

    #[tx]
    pub async fn block_user(
        &self,
        session: &mut ClientSession,
        tg_id: i64,
        is_active: bool,
    ) -> Result<()> {
        info!("Blocking user: {}", tg_id);
        warn!("remove subscription!!!!");
        self.store.block(session, tg_id, is_active).await?;
        Ok(())
    }

    pub async fn edit_user_rule(
        &self,
        session: &mut ClientSession,
        tg_id: i64,
        rule: Rule,
        is_active: bool,
    ) -> Result<()> {
        if is_active {
            self.store.add_rule(session, tg_id, &rule).await?;
            info!("Adding rule {:?} to user {}", rule, tg_id);
        } else {
            self.store.remove_rule(session, tg_id, &rule).await?;
            info!("Removing rule {:?} from user {}", rule, tg_id);
        }

        Ok(())
    }

    pub(crate) async fn increment_balance(
        &self,
        session: &mut ClientSession,
        tg_id: i64,
        amount: u32,
    ) -> Result<(), Error> {
        self.store.increment_balance(session, tg_id, amount).await?;
        Ok(())
    }
}

pub enum SetDateError {
    UserNotFound,
    AlreadySet,
    Common(eyre::Error),
}
