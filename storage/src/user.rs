pub mod rights;

use crate::Storage;
use chrono::{DateTime, Local};
use eyre::Result;
use mongodb::bson::doc;
use rights::Rights;
use serde::{Deserialize, Serialize};

pub(crate) const COLLECTION: &str = "Users";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub chat_id: i64,
    pub user_id: String,
    pub name: UserName,
    pub rights: Rights,
    pub phone: String,
    pub birthday: Option<DateTime<Local>>,
    pub reg_date: DateTime<Local>,
    pub balance: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserName {
    pub tg_user_name: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
}

impl Storage {
    pub async fn get_user_by_id(&self, id: String) -> Result<Option<User>> {
        Ok(self.users.find_one(doc! { "user_id": id }).await?)
    }

    pub async fn insert_user(&self, user: User) -> Result<()> {
        self.users.insert_one(user).await?;
        Ok(())
    }

    pub async fn users_count(&self) -> Result<u64> {
        Ok(self.users.count_documents(doc! {}).await?)
    }
}
