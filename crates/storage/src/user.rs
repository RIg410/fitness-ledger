use bson::{to_document, Document};
use chrono::{DateTime, Duration, Local, Utc};
use eyre::{bail, eyre, Error, Result};
use futures_util::stream::TryStreamExt;
use log::info;
use model::couch::CouchInfo;
use model::rights;
use model::session::Session;
use model::subscription::{Status, Subscription, UserSubscription};
use model::user::{Freeze, Notification, User, UserIdent};
use mongodb::bson::to_bson;
use mongodb::options::UpdateOptions;
use mongodb::IndexModel;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection, Database,
};
use std::sync::Arc;

const COLLECTION: &str = "users";

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

    pub async fn get<ID: Into<UserIdent>>(
        &self,
        session: &mut Session,
        id: ID,
    ) -> Result<Option<User>> {
        Ok(self
            .users
            .find_one(self.ident_filter(id))
            .session(&mut *session)
            .await?)
    }

    pub async fn find_by_phone(&self, session: &mut Session, phone: &str) -> Result<Option<User>> {
        Ok(self
            .users
            .find_one(doc! { "phone": phone })
            .session(&mut *session)
            .await?)
    }

    fn ident_filter<ID: Into<UserIdent>>(&self, id: ID) -> Document {
        match id.into() {
            UserIdent::TgId(tg_id) => doc! { "tg_id": tg_id },
            UserIdent::Id(id) => doc! { "_id": id },
        }
    }

    pub async fn insert(&self, session: &mut Session, user: User) -> Result<()> {
        info!("Inserting user: {:?}", user);
        let result = self
            .users
            .update_one(
                doc! { "tg_id": user.tg_id },
                doc! { "$setOnInsert": to_document(&user)? },
            )
            .session(&mut *session)
            .with_options(UpdateOptions::builder().upsert(true).build())
            .await?;
        if result.upserted_id.is_none() {
            return Err(Error::msg("User already exists"));
        }

        Ok(())
    }

    pub async fn count(&self, session: &mut Session) -> Result<u64> {
        Ok(self
            .users
            .count_documents(doc! {})
            .session(&mut *session)
            .await?)
    }

    pub async fn find(
        &self,
        session: &mut Session,
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
        let mut cursor = self
            .users
            .find(query)
            .skip(offset)
            .limit(limit as i64)
            .session(&mut *session)
            .await?;
        Ok(cursor.stream(&mut *session).try_collect().await?)
    }

    pub async fn add_subscription(
        &self,
        session: &mut Session,
        tg_id: i64,
        sub: Subscription,
    ) -> Result<()> {
        info!("Add subscription for user {}: {:?}", tg_id, sub);
        let freeze_days = sub.freeze_days as i32;
        let amount = sub.items as i32;

        let result = self
            .users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! {
                "$inc": {
                    "balance": amount,
                    "freeze_days": freeze_days,
                     "version": 1
                    },
                    "$push": {
                        "subscriptions": to_document(&UserSubscription::from(sub))?
                    }
                },
            )
            .session(&mut *session)
            .await?;

        if result.modified_count != 1 {
            return Err(eyre!("Failed to modify balance"));
        }
        Ok(())
    }

    pub async fn find_users_to_unfreeze(&self, session: &mut Session) -> Result<Vec<User>, Error> {
        let filter = doc! {
            "freeze.freeze_end": { "$lte": Local::now().with_timezone(&Utc) }
        };
        let mut cursor = self.users.find(filter).session(&mut *session).await?;
        Ok(cursor.stream(&mut *session).try_collect().await?)
    }

    pub async fn unfreeze(&self, session: &mut Session, tg_id: i64) -> Result<()> {
        info!("Unfreeze account:{}", tg_id);
        let result = self
            .users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$unset": { "freeze": "" }, "$inc": { "version": 1 } },
            )
            .session(&mut *session)
            .await?;

        if result.modified_count != 1 {
            return Err(eyre!("Failed to unfreeze account"));
        }
        Ok(())
    }

    pub async fn freeze<ID: Into<UserIdent>>(
        &self,
        session: &mut Session,
        id: ID,
        days: u32,
    ) -> Result<()> {
        let id = id.into();
        info!("Freeze account:{}", id);
        let mut user = self
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found:{}", id))?;
        user.version += 1;

        if user.freeze_days < days {
            bail!("Insufficient freeze days");
        }
        user.freeze_days -= days;

        for sub in user.subscriptions.iter_mut() {
            match sub.status {
                Status::NotActive => {
                    //no-op
                }
                Status::Active { start_date } => {
                    sub.status = Status::Active {
                        start_date: start_date + Duration::days(days as i64),
                    }
                }
            }
        }
        user.freeze = Some(Freeze {
            freeze_start: Local::now().with_timezone(&Utc),
            freeze_end: Local::now().with_timezone(&Utc) + chrono::Duration::days(days as i64),
        });

        self.users
            .update_one(self.ident_filter(id), doc! { "$set": to_document(&user)? })
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn reserve_balance(
        &self,
        session: &mut Session,
        tg_id: i64,
        amount: u32,
        sign_up_date: DateTime<Utc>,
    ) -> Result<()> {
        info!("Reserving balance for user {}: {}", tg_id, amount);
        let mut user = self
            .get(session, tg_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        user.version += 1;
        if user.balance < amount {
            bail!("Insufficient balance");
        }
        user.balance -= amount;
        user.reserved_balance += amount;

        let has_active = user.subscriptions.iter().any(|s| s.is_active());
        if !has_active {
            user.subscriptions.sort_by(|a, b| a.status.cmp(&b.status));
            if let Some(sub) = user.subscriptions.first_mut() {
                sub.activate(sign_up_date);
            }
        }

        let updated = self
            .users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": to_document(&user)? },
            )
            .session(&mut *session)
            .await?;

        if updated.modified_count != 1 {
            return Err(Error::msg("User not found or insufficient balance"));
        }
        Ok(())
    }

    pub async fn charge_reserved_balance(
        &self,
        session: &mut Session,
        tg_id: i64,
        amount: u32,
    ) -> Result<()> {
        info!("Charging blocked balance for user {}: {}", tg_id, amount);
        let mut user = self
            .get(session, tg_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        if user.reserved_balance < amount {
            bail!("Insufficient reserved balance");
        }
        user.version += 1;
        user.reserved_balance = user.reserved_balance.saturating_sub(amount);

        let active = user.subscriptions.iter_mut().find(|s| s.is_active());
        if let Some(sub) = active {
            sub.items = sub.items.saturating_sub(amount);
        }
        user.subscriptions.retain(|sub| sub.items > 0);

        self.users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": to_document(&user)? },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn unblock_balance(
        &self,
        session: &mut Session,
        tg_id: i64,
        amount: u32,
    ) -> Result<()> {
        info!("Unblocking balance for user {}: {}", tg_id, amount);
        let amount = amount as i32;
        let result = self
            .users
            .update_one(
                doc! { "tg_id": tg_id, "reserved_balance": { "$gte": amount } },
                doc! { "$inc": { "reserved_balance": -amount, "balance": amount , "version": 1} },
            )
            .session(&mut *session)
            .await?;
        if result.modified_count == 0 {
            return Err(Error::msg(
                "User not found or insufficient reserved_balance",
            ));
        }
        Ok(())
    }

    pub async fn set_first_name(
        &self,
        session: &mut Session,
        tg_id: i64,
        first_name: &str,
    ) -> Result<bool> {
        info!("Setting first_name for user {}: {}", tg_id, first_name);
        let result = self
            .users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": { "name.first_name": first_name }, "$inc": { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn set_last_name(
        &self,
        session: &mut Session,
        tg_id: i64,
        last_name: &str,
    ) -> Result<bool> {
        info!("Setting last_name for user {}: {}", tg_id, last_name);
        let result = self
            .users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": { "name.last_name": last_name }, "$inc": { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn set_tg_user_name(
        &self,
        session: &mut Session,
        tg_id: i64,
        tg_user_name: &str,
    ) -> Result<bool> {
        info!("Setting tg_user_name for user {}: {}", tg_id, tg_user_name);
        let result = self
            .users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": { "name.tg_user_name": tg_user_name }, "$inc": { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn instructors(&self, session: &mut Session) -> Result<Vec<User>, Error> {
        let filter = doc! { "couch": { "$exists": true, "$ne": null } };
        let mut cursor = self.users.find(filter).session(&mut *session).await?;
        Ok(cursor.stream(&mut *session).try_collect().await?)
    }

    pub async fn set_birthday(
        &self,
        session: &mut Session,
        tg_id: i64,
        birthday: DateTime<Local>,
    ) -> Result<bool> {
        info!("Setting birthday for user {}: {}", tg_id, birthday);
        let result = self
            .users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": { "birthday": to_bson(&birthday)? }, "$inc": { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn block_user(
        &self,
        session: &mut Session,
        tg_id: i64,
        is_active: bool,
    ) -> Result<bool> {
        info!("Blocking user {}: {}", tg_id, is_active);
        let result = self
            .users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": { "is_active": is_active }, "$inc": { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn add_rule(
        &self,
        session: &mut Session,
        tg_id: i64,
        rule: &rights::Rule,
    ) -> Result<bool> {
        info!("Adding rule {:?} to user {}", rule, tg_id);
        let result = self.users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$addToSet": { "rights.rights": format!("{:?}", rule) }, "$inc": { "version": 1 } },
            ).session(&mut *session)
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn remove_rule(
        &self,
        session: &mut Session,
        tg_id: i64,
        rule: &rights::Rule,
    ) -> Result<bool> {
        info!("Removing rule {:?} from user {}", rule, tg_id);
        let result = self.users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$pull": { "rights.rights": format!("{:?}", rule) }, "$inc": { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn change_reserved_balance(
        &self,
        session: &mut Session,
        tg_id: i64,
        amount: i32,
    ) -> Result<()> {
        info!("Changing reserved balance for user {}: {}", tg_id, amount);
        let mut user = self
            .get(session, tg_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        user.version += 1;
        if amount < 0 {
            user.reserved_balance = user.reserved_balance.saturating_sub(amount.unsigned_abs());
        } else {
            user.reserved_balance += amount as u32;
        }

        self.users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": to_document(&user)? },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn change_balance(
        &self,
        session: &mut Session,
        tg_id: i64,
        amount: i32,
    ) -> Result<()> {
        info!("Changing balance for user {}: {}", tg_id, amount);
        let mut user = self
            .get(session, tg_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        user.version += 1;
        if amount < 0 {
            user.balance = user.balance.saturating_sub(amount.unsigned_abs());
        } else {
            user.balance += amount as u32;
        }

        user.subscriptions.sort_by(|a, b| a.status.cmp(&b.status));
        if let Some(sub) = user.subscriptions.first_mut() {
            if amount < 0 {
                sub.items = sub.items.saturating_sub(amount.unsigned_abs());
            } else {
                sub.items += amount as u32;
            }
        }
        user.subscriptions.retain(|sub| sub.items > 0);

        self.users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": to_document(&user)? },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn find_subscription_to_expire(
        &self,
        session: &mut Session,
    ) -> Result<Vec<User>, Error> {
        let filter = doc! {
            "subscriptions.end_date": { "$lte": Local::now().with_timezone(&Utc) }
        };
        let mut cursor = self.users.find(filter).session(&mut *session).await?;
        Ok(cursor.stream(&mut *session).try_collect().await?)
    }

    pub async fn expire_subscription<ID: Into<UserIdent>>(
        &self,
        session: &mut Session,
        id: ID,
    ) -> Result<Vec<UserSubscription>> {
        let id = id.into();
        let now = Local::now().with_timezone(&Utc);
        info!("Expire subscription for user {}", id);
        let mut user = self
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        user.version += 1;

        user.subscriptions
            .iter()
            .filter(|sub| sub.is_expired(now))
            .for_each(|sub| {
                user.balance -= sub.items;
            });

        let (expired, actual) = user.subscriptions.into_iter().fold(
            (Vec::new(), Vec::new()),
            |(mut expired, mut actual), sub| {
                if sub.is_expired(now) {
                    expired.push(sub);
                } else {
                    actual.push(sub);
                }
                (expired, actual)
            },
        );

        user.subscriptions = actual;
        self.users
            .update_one(self.ident_filter(id), doc! { "$set": to_document(&user)? })
            .session(&mut *session)
            .await?;
        Ok(expired)
    }

    pub async fn set_phone(&self, session: &mut Session, tg_id: i64, phone: &str) -> Result<()> {
        info!("Setting phone for user {}: {}", tg_id, phone);
        let result = self
            .users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": { "phone": phone }, "$inc": { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        if result.modified_count == 0 {
            return Err(Error::msg("User not found"));
        }
        Ok(())
    }

    pub async fn get_by_phone(
        &self,
        session: &mut Session,
        phone: &str,
    ) -> std::result::Result<Option<User>, Error> {
        Ok(self
            .users
            .find_one(doc! { "phone": phone })
            .session(&mut *session)
            .await?)
    }

    pub async fn set_couch(
        &self,
        session: &mut Session,
        tg_id: i64,
        couch: &CouchInfo,
    ) -> Result<()> {
        info!("Setting couch for user {}: {:?}", tg_id, couch);
        self.users
            .update_one(
                doc! { "tg_id": tg_id },
                doc! { "$set": { "couch": to_document(couch)? }, "$inc": { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn delete_couch(&self, session: &mut Session, id: ObjectId) -> Result<(), Error> {
        info!("Deleting couch: {:?}", id);
        let result = self
            .users
            .update_one(
                doc! { "_id": id },
                doc! { "$unset": { "couch": "" }, "$inc": { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        if result.modified_count == 0 {
            return Err(Error::msg("Couch not found"));
        }
        Ok(())
    }

    pub async fn update_couch_reward(
        &self,
        session: &mut Session,
        id: ObjectId,
        reward: model::decimal::Decimal,
    ) -> std::result::Result<(), Error> {
        info!("Updating couch reward: {:?}", id);
        let result = self
            .users
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "couch.reward":  reward.inner() }, "$inc": { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        if result.modified_count == 0 {
            return Err(Error::msg("Couch not found"));
        }
        Ok(())
    }

    pub async fn update_notification_settings(
        &self,
        session: &mut Session,
        id: ObjectId,
        settings: Notification,
    ) -> Result<(), Error> {
        info!("Updating notification settings: {:?}", settings);
        self.users
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "settings.notification": to_document(&settings)? }, "$inc": { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn dump(&self, session: &mut Session) -> Result<Vec<User>> {
        let mut cursor = self.users.find(doc! {}).session(&mut *session).await?;
        Ok(cursor.stream(&mut *session).try_collect().await?)
    }
}
