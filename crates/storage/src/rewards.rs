use bson::{doc, oid::ObjectId};
use eyre::Error;
use model::{couch::Reward, session::Session};
use mongodb::Collection;

const REWARD_COLLECTION: &str = "reward";

pub struct RewardsStore {
    rewards: Collection<Reward>,
}

impl RewardsStore {
    pub async fn new(db: &mongodb::Database) -> Result<Self, Error> {
        let rewards = db.collection(REWARD_COLLECTION);
        Ok(RewardsStore { rewards })
    }

    pub async fn add_reward(&self, session: &mut Session, reward: Reward) -> Result<(), Error> {
        self.rewards
            .insert_one(reward)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, session: &mut Session, reward: Reward) -> Result<(), Error> {
        self.rewards
            .delete_one(doc! {"_id": reward.id})
            .session(session)
            .await?;
        Ok(())
    }

    pub async fn get(
        &self,
        session: &mut Session,
        couch_id: ObjectId,
        limit: i64,
        offset: u64,
    ) -> Result<Vec<Reward>, Error> {
        let mut cursor = self
            .rewards
            .find(doc! {
                "couch": couch_id
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

    pub async fn dump(&self, session: &mut Session) -> Result<Vec<Reward>, Error> {
        let mut cursor = self.rewards.find(doc! {}).session(&mut *session).await?;

        let mut rewards = Vec::new();
        while let Some(reward) = cursor.next(&mut *session).await {
            rewards.push(reward?);
        }
        Ok(rewards)
    }
}
