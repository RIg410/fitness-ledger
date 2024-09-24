use eyre::Error;
use model::{couch::Reward, session::Session};
use mongodb::bson::oid::ObjectId;
use storage::rewards::RewardsStore;

#[derive(Clone)]
pub struct Rewards {
    store: RewardsStore,
}

impl Rewards {
    pub(crate) fn new(store: RewardsStore) -> Self {
        Rewards { store }
    }

    pub async fn add_reward(&self, session: &mut Session, reward: Reward) -> Result<(), Error> {
        self.store.add_reward(session, reward).await?;
        Ok(())
    }

    pub async fn rewards(
        &self,
        session: &mut Session,
        couch_id: ObjectId,
        limit: i64,
        offset: u64,
    ) -> Result<Vec<Reward>, Error> {
        self.store
            .get_rewards(session, couch_id, limit, offset)
            .await
    }
}
