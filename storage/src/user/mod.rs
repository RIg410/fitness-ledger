pub mod rights;
pub mod stat;

use crate::date_time::{opt_naive_date_deserialize, opt_naive_date_serialize, Date};
use crate::Storage;
use chrono::{DateTime, Local, NaiveDate};
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
    #[serde(serialize_with = "opt_naive_date_serialize")]
    #[serde(deserialize_with = "opt_naive_date_deserialize")]
    pub birthday: Option<NaiveDate>,
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
    pub async fn get_user_by_chat_id(&self, chat_id: i64) -> Result<Option<User>> {
        Ok(self.users.find_one(doc! { "chat_id": chat_id }).await?)
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        Ok(self.users.find_one(doc! { "user_id": id }).await?)
    }

    pub async fn insert_user(&self, user: User) -> Result<()> {
        self.users.insert_one(user).await?;
        Ok(())
    }

    pub async fn users_count(&self) -> Result<u64> {
        Ok(self.users.count_documents(doc! {}).await?)
    }

    pub async fn set_user_birthday(&self, id: &str, birthday: NaiveDate) -> Result<()> {
        let date = mongodb::bson::to_document(&Date::from(birthday))?;
        self.users
            .update_one(
                doc! { "user_id": id },
                doc! { "$set": { "birthday": date } },
            )
            .await?;
        Ok(())
    }

    pub async fn update_chat_id(&self, id: &str, chat_id: i64) -> Result<()> {
        self.users
            .update_one(doc! { "user_id": id }, doc! { "$set": { "chat_id": chat_id } })
            .await?;
        Ok(())
    }
}
