use eyre::Result;
use log::info;
use storage::{
    user::{
        rights::{Rights, Rule, TrainingRule, UserRule},
        User, UserName, UserStore,
    },
    Storage,
};

#[derive(Clone)]
pub struct Ledger {
    storage: UserStore,
}

impl Ledger {
    pub fn new(storage: Storage) -> Self {
        Ledger {
            storage: storage.users,
        }
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        self.storage.get_user_by_id(id).await
    }

    pub async fn create_user(
        &self,
        chat_id: i64,
        user_id: String,
        name: UserName,
        phone: String,
    ) -> Result<()> {
        let is_first_user = self.storage.users_count().await? == 0;
        let mut rights = Rights::default();
        if is_first_user {
            rights.add_rule(Rule::Full);
        } else {
            rights.add_rule(Rule::User(UserRule::ViewSelfProfile));
            rights.add_rule(Rule::User(UserRule::EditSelfProfile));
            rights.add_rule(Rule::Training(TrainingRule::SignupForTraining));
            rights.add_rule(Rule::Training(TrainingRule::CancelTrainingSignup));
            rights.add_rule(Rule::Training(TrainingRule::ViewSchedule));
            rights.add_rule(Rule::Subscription(
                storage::user::rights::SubscriptionsRule::ViewSubscription,
            ));
        }

        let user = User {
            chat_id,
            user_id,
            name,
            rights,
            phone,
            birthday: None,
            reg_date: chrono::Local::now(),
            balance: 0,
        };
        info!("Creating user: {:?}", user);
        self.storage.insert_user(user).await?;
        Ok(())
    }

    pub async fn user_count(&self) -> Result<u64> {
        self.storage.users_count().await
    }

    pub async fn find_users(&self, query: &str, limit: u64, offset: u64) -> Result<Vec<User>> {
        let keywords = query.split_whitespace().collect::<Vec<_>>();
        self.storage.find_users(&keywords, limit, offset).await
    }

    pub async fn set_user_birthday(
        &self,
        id: &str,
        date: chrono::NaiveDate,
    ) -> std::result::Result<(), SetDateError> {
        let user = self
            .storage
            .get_user_by_id(id)
            .await
            .map_err(|err| SetDateError::Common(err))?;
        let user = user.ok_or(SetDateError::UserNotFound)?;
        if user.birthday.is_some() {
            return Err(SetDateError::AlreadySet);
        }
        self.storage
            .set_user_birthday(&user.user_id, date)
            .await
            .map_err(|err| SetDateError::Common(err))?;
        Ok(())
    }

    pub async fn update_chat_id(&self, id: &str, chat_id: i64) -> Result<()> {
        self.storage.update_chat_id(id, chat_id).await
    }
}

pub enum SetDateError {
    UserNotFound,
    AlreadySet,
    Common(eyre::Error),
}
