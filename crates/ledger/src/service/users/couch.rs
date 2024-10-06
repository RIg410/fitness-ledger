use eyre::{bail, eyre, Result};
use model::{
    couch::{CouchInfo, Rate},
    decimal::Decimal,
    session::Session,
};
use mongodb::bson::oid::ObjectId;
use tx_macro::tx;

use super::Users;

impl Users {
    #[tx]
    pub async fn update_couch_rate(
        &self,
        session: &mut Session,
        id: ObjectId,
        rate: Rate,
    ) -> Result<()> {
        self.update_couch_rate_tx_less(session, id, rate).await
    }

    pub async fn update_couch_rate_tx_less(
        &self,
        session: &mut Session,
        id: ObjectId,
        rate: Rate,
    ) -> Result<()> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        let couch = user.couch.ok_or_else(|| eyre!("User is not a couch"))?;
        let couch = CouchInfo {
            description: couch.description,
            rate,
            reward: couch.reward,
        };
        self.store.set_couch(session, user.tg_id, &couch).await?;
        Ok(())
    }

    #[tx]
    pub async fn update_couch_description(
        &self,
        session: &mut Session,
        id: ObjectId,
        description: String,
    ) -> Result<()> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        let couch = user.couch.ok_or_else(|| eyre!("User is not a couch"))?;
        let couch = CouchInfo {
            description: description.clone(),
            rate: couch.rate,
            reward: couch.reward,
        };
        self.store.set_couch(session, user.tg_id, &couch).await?;
        Ok(())
    }

    #[tx]
    pub async fn make_user_couch(
        &self,
        session: &mut Session,
        tg_id: i64,
        description: String,
        rate: Rate,
    ) -> Result<()> {
        let user = self
            .store
            .get(session, tg_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        if user.couch.is_some() {
            bail!("Already instructor");
        }

        let couch = CouchInfo {
            description,
            reward: Decimal::zero(),
            rate,
        };
        self.store.set_couch(session, tg_id, &couch).await?;
        Ok(())
    }
}
