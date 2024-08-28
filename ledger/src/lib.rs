use std::sync::Arc;

use eyre::Result;
use log::info;
use storage::{
    user::{
        rights::{Rights, Rule, TrainingRule, UserRule},
        User, UserName,
    },
    Storage,
};

#[derive(Clone)]
pub struct Ledger {
    storage: Arc<Storage>,
}

impl Ledger {
    pub fn new(storage: Storage) -> Self {
        Ledger {
            storage: Arc::new(storage),
        }
    }

    pub async fn get_user_by_id(&self, id: String) -> Result<Option<User>> {
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
}
