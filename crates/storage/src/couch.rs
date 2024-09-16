use std::sync::Arc;

use bson::{doc, oid::ObjectId, to_document};
use eyre::Error;
use model::{
    couch::{Couch, Reward},
    session::Session,
};
use mongodb::Collection;

const COLLECTION: &str = "couch";
const REWARD_COLLECTION: &str = "couch_reward";

#[derive(Clone)]
pub struct TreasuryStore {
    store: Arc<Collection<Couch>>,
    rewards: Arc<Collection<Reward>>,
}

impl TreasuryStore {
    pub async fn new(db: &mongodb::Database) -> Result<Self, Error> {
        let store = db.collection(COLLECTION);
        let reward = db.collection(REWARD_COLLECTION);
        Ok(TreasuryStore {
            store: Arc::new(store),
            rewards: Arc::new(reward),
        })
    }

    pub async fn insert(&self, session: &mut Session, couch_info: &Couch) -> Result<(), Error> {
        self.store.insert_one(couch_info).session(session).await?;
        Ok(())
    }

    pub async fn get(&self, session: &mut Session, id: ObjectId) -> Result<Option<Couch>, Error> {
        Ok(self
            .store
            .find_one(doc! {
                "_id": id
            })
            .session(session)
            .await?)
    }

    pub async fn update(&self, session: &mut Session, couch_info: &Couch) -> Result<(), Error> {
        self.store
            .update_one(
                doc! { "_id": couch_info.id },
                doc! { "$set": to_document(&couch_info)? },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn add_reward(&self, session: &mut Session, reward: Reward) -> Result<(), Error> {
        self.rewards
            .insert_one(reward)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn get_rewards(
        &self,
        session: &mut Session,
        couch_id: ObjectId,
        limit: i64,
        offset: u64,
    ) -> Result<Vec<Reward>, Error> {
        let mut cursor = self
            .rewards
            .find(doc! {
                "couch_id": couch_id
            })
            .skip(offset)
            .limit(limit)
            .sort(doc! { "created_at": -1 })
            .session(&mut *session)
            .await?;

        let mut rewards = Vec::with_capacity(limit as usize);
        while let Some(reward) = cursor.next(&mut *session).await {
            rewards.push(reward?);
        }
        Ok(rewards)
    }
}
