pub mod model;
pub mod rights;
pub mod stat;
use std::sync::Arc;

pub use model::{User, UserName};

use crate::date_time::Date;
use chrono::NaiveDate;
use eyre::Result;
use futures_util::stream::TryStreamExt;
use mongodb::{bson::doc, Collection, Database};

const COLLECTION: &str = "Users";

#[derive(Clone)]
pub struct UserStore {
    pub(crate) users: Arc<Collection<User>>,
}

impl UserStore {
    pub(crate) fn new(db: &Database) -> Self {
        let users = db.collection(COLLECTION);
        UserStore {
            users: Arc::new(users),
        }
    }

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
            .update_one(
                doc! { "user_id": id },
                doc! { "$set": { "chat_id": chat_id } },
            )
            .await?;
        Ok(())
    }

    pub async fn find_users(
        &self,
        keywords: &[&str],
        offset: u64,
        limit: u64,
    ) -> Result<Vec<User>> {
        let mut query = doc! {};
        if !keywords.is_empty() {
            let mut keyword_query = vec![];
            for keyword in keywords {
                let regex = format!("^{}", keyword);
                let regex_query = doc! {
                    "$or": [
                        { "name.first_name": { "$regex": &regex, "$options": "i" } },
                        { "name.last_name": { "$regex": &regex, "$options": "i" } },
                        { "name.tg_user_name": { "$regex": &regex, "$options": "i" } },
                        { "phone": { "$regex": &regex, "$options": "i" } },
                    ]
                };
                keyword_query.push(regex_query);
            }
            query = doc! { "$or": keyword_query };
        }
        let cursor = self
            .users
            .find(query)
            .skip(offset)
            .limit(limit as i64)
            .await?;
        Ok(cursor.try_collect().await?)
    }

    pub async fn block_user(&self, user_id: &str, is_active: bool) -> Result<()> {
        self.users
            .update_one(
                doc! { "user_id": user_id },
                doc! { "$set": { "is_active": is_active } },
            )
            .await?;
        Ok(())
    }

    pub async fn add_user_rule(&self, user_id: &str, rule: &rights::Rule) -> Result<()> {
        let rule = mongodb::bson::to_document(&rule)?;
        self.users
            .update_one(
                doc! { "user_id": user_id },
                doc! { "$addToSet": { "rights.rights": rule } },
            )
            .await?;
        Ok(())
    }

    pub async fn remove_user_rule(&self, user_id: &str, rule: &rights::Rule) -> Result<()> {
        let rule = mongodb::bson::to_document(&rule)?;

        self.users
            .update_one(
                doc! { "user_id": user_id },
                doc! { "$pull": { "rights.rights": rule } },
            )
            .await?;
        Ok(())
    }
}
