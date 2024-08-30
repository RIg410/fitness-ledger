pub mod model;
pub mod rights;
pub mod stat;
use std::sync::Arc;

pub use model::{User, UserName};

use crate::date_time::Date;
use chrono::NaiveDate;
use eyre::Result;
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection, Database,
};

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

    pub async fn get_by_tg_id(&self, tg_id: i64) -> Result<Option<User>> {
        Ok(self.users.find_one(doc! { "tg_id": tg_id }).await?)
    }

    pub async fn get_by_id(&self, id: ObjectId) -> Result<Option<User>> {
        Ok(self.users.find_one(doc! { "_id": id }).await?)
    }

    pub async fn insert(&self, user: User) -> Result<()> {
        self.users.insert_one(user).await?;
        Ok(())
    }

    pub async fn count(&self) -> Result<u64> {
        Ok(self.users.count_documents(doc! {}).await?)
    }

    pub async fn set_birthday(&self, tg_id: i64, birthday: NaiveDate) -> Result<()> {
        let date = mongodb::bson::to_document(&Date::from(birthday))?;
        self.users
            .update_one(doc! { "tg_id": tg_id }, doc! { "$set": { "birthday": date } })
            .await?;
        Ok(())
    }

    pub async fn find(&self, keywords: &[&str], offset: u64, limit: u64) -> Result<Vec<User>> {
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

    pub async fn block(&self, tg_id: i64, is_active: bool) -> Result<()> {
        self.users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": { "is_active": is_active } },
            )
            .await?;
        Ok(())
    }

    pub async fn add_rule(&self, tg_id: i64, rule: &rights::Rule) -> Result<()> {
        self.users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$addToSet": { "rights.rights": format!("{:?}", rule) } },
            )
            .await?;
        Ok(())
    }

    pub async fn remove_rule(&self, tg_id: i64, rule: &rights::Rule) -> Result<()> {
        self.users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$pull": { "rights.rights": format!("{:?}", rule) } },
            )
            .await?;
        Ok(())
    }
}
