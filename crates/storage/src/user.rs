use bson::to_document;
use chrono::{DateTime, Local};
use eyre::{Error, Result};
use futures_util::stream::TryStreamExt;
use log::info;
use model::rights;
use model::user::User;
use mongodb::bson::to_bson;
use mongodb::options::UpdateOptions;
use mongodb::IndexModel;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection, Database,
};
use std::sync::Arc;

const COLLECTION: &str = "Users";

#[derive(Clone)]
pub struct UserStore {
    pub(crate) users: Arc<Collection<User>>,
}

impl UserStore {
    pub(crate) async fn new(db: &Database) -> Result<Self> {
        let users = db.collection(COLLECTION);
        users
            .create_index(IndexModel::builder().keys(doc! { "tg_id": 1 }).build())
            .await?;
        Ok(UserStore {
            users: Arc::new(users),
        })
    }

    pub async fn get_by_tg_id(&self, tg_id: i64) -> Result<Option<User>> {
        Ok(self.users.find_one(doc! { "tg_id": tg_id }).await?)
    }

    pub async fn get_by_id(&self, id: ObjectId) -> Result<Option<User>> {
        Ok(self.users.find_one(doc! { "_id": id }).await?)
    }

    pub async fn insert(&self, user: User) -> Result<()> {
        info!("Inserting user: {:?}", user);
        let result = self
            .users
            .update_one(
                doc! { "tg_id": user.tg_id },
                doc! { "$setOnInsert": to_document(&user)? },
            )
            .with_options(UpdateOptions::builder().upsert(true).build())
            .await?;
        if result.upserted_id.is_none() {
            return Err(Error::msg("User already exists"));
        }

        Ok(())
    }

    pub async fn count(&self) -> Result<u64> {
        Ok(self.users.count_documents(doc! {}).await?)
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

    pub async fn increment_balance(&self, tg_id: i64, amount: u32) -> Result<()> {
        info!("Incrementing balance for user {}: {}", tg_id, amount);
        let amount = amount as i32;
        self.users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$inc": { "balance": amount }, "$inc": { "version": 1 } },
            )
            .await?;
        Ok(())
    }

    pub async fn reserve_balance(&self, tg_id: i64, amount: u32) -> Result<()> {
        info!("Reserving balance for user {}: {}", tg_id, amount);
        let amount = amount as i32;
        let updated = self.users
                .update_one(
                    doc! { "tg_id": tg_id, "balance": { "$gte": amount } },
                    doc! { "$inc": { "balance": -amount, "reserved_balance": amount }, "$inc": { "version": 1 } },
                )
                .await?;

        if updated.modified_count == 0 {
            return Err(Error::msg("User not found or insufficient balance"));
        }
        Ok(())
    }

    pub async fn charge_reserved_balance(&self, tg_id: i64, amount: u32) -> Result<()> {
        info!("Charging blocked balance for user {}: {}", tg_id, amount);
        let amount = amount as i32;
        self.users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$inc": { "reserved_balance": -amount }, "$inc": { "version": 1 } },
            )
            .await?;
        Ok(())
    }

    pub async fn unblock_balance(&self, tg_id: i64, amount: u32) -> Result<()> {
        info!("Unblocking balance for user {}: {}", tg_id, amount);
        let amount = amount as i32;
        let result = self.users
            .update_one(
                doc! { "tg_id": tg_id, "reserved_balance": { "$gte": amount } },
                doc! { "$inc": { "reserved_balance": -amount, "balance": amount }, "$inc": { "version": 1 } },
            )
            .await?;
        if result.modified_count == 0 {
            return Err(Error::msg("User not found or insufficient reserved_balance"));
        }
        Ok(())
    }

    pub async fn set_first_name(&self, tg_id: i64, first_name: &str) -> Result<bool> {
        info!("Setting first_name for user {}: {}", tg_id, first_name);
        let result = self
            .users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": { "name.first_name": first_name }, "$inc": { "version": 1 } },
            )
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn set_last_name(&self, tg_id: i64, last_name: &str) -> Result<bool> {
        info!("Setting last_name for user {}: {}", tg_id, last_name);
        let result = self
            .users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": { "name.last_name": last_name }, "$inc": { "version": 1 } },
            )
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn set_tg_user_name(&self, tg_id: i64, tg_user_name: &str) -> Result<bool> {
        info!("Setting tg_user_name for user {}: {}", tg_id, tg_user_name);
        let result = self
            .users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": { "name.tg_user_name": tg_user_name }, "$inc": { "version": 1 } },
            )
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn get_instructors(&self) -> Result<Vec<User>, Error> {
        let filter = doc! { "rights.rights": "Train" };
        let cursor = self.users.find(filter).await?;
        Ok(cursor.try_collect().await?)
    }

    pub async fn set_birthday(&self, tg_id: i64, birthday: DateTime<Local>) -> Result<bool> {
        info!("Setting birthday for user {}: {}", tg_id, birthday);
        let result = self
            .users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": { "birthday": to_bson(&birthday)? }, "$inc": { "version": 1 } },
            )
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn block(&self, tg_id: i64, is_active: bool) -> Result<bool> {
        info!("Blocking user {}: {}", tg_id, is_active);
        let result = self
            .users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": { "is_active": is_active }, "$inc": { "version": 1 } },
            )
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn add_rule(&self, tg_id: i64, rule: &rights::Rule) -> Result<bool> {
        info!("Adding rule {:?} to user {}", rule, tg_id);
        let result = self.users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$addToSet": { "rights.rights": format!("{:?}", rule) }, "$inc": { "version": 1 } },
            )
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn remove_rule(&self, tg_id: i64, rule: &rights::Rule) -> Result<bool> {
        info!("Removing rule {:?} from user {}", rule, tg_id);
        let result = self.users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$pull": { "rights.rights": format!("{:?}", rule) }, "$inc": { "version": 1 } },
            )
            .await?;
        Ok(result.modified_count > 0)
    }
}
