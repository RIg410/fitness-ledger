use eyre::{bail, eyre, Result};
use model::{
    couch::{CouchInfo, GroupRate, PersonalRate},
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
        rate: GroupRate,
    ) -> Result<()> {
        self.update_couch_rate_tx_less(session, id, rate).await
    }

    #[tx]
    pub async fn update_couch_personal_rate(
        &self,
        session: &mut Session,
        id: ObjectId,
        personal_rate: PersonalRate,
    ) -> Result<()> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        let couch = user.couch.ok_or_else(|| eyre!("User is not a couch"))?;
        let couch = CouchInfo {
            description: couch.description,
            group_rate: couch.group_rate,
            reward: couch.reward,
            personal_rate,
        };
        self.store.set_couch(session, user.id, &couch).await?;
        Ok(())
    }

    pub async fn update_couch_rate_tx_less(
        &self,
        session: &mut Session,
        id: ObjectId,
        rate: GroupRate,
    ) -> Result<()> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        let couch = user.couch.ok_or_else(|| eyre!("User is not a couch"))?;
        let couch = CouchInfo {
            description: couch.description,
            group_rate: rate,
            reward: couch.reward,
            personal_rate: couch.personal_rate,
        };
        self.store.set_couch(session, user.id, &couch).await?;
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
            group_rate: couch.group_rate,
            reward: couch.reward,
            personal_rate: couch.personal_rate,
        };
        self.store.set_couch(session, user.id, &couch).await?;
        Ok(())
    }

    #[tx]
    pub async fn make_user_couch(
        &self,
        session: &mut Session,
        id: ObjectId,
        description: String,
        group_rate: GroupRate,
        personal_rate: PersonalRate,
    ) -> Result<()> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        if user.couch.is_some() {
            bail!("Already instructor");
        }

        let couch = CouchInfo {
            description,
            reward: Decimal::zero(),
            group_rate,
            personal_rate,
        };
        self.store.set_couch(session, id, &couch).await?;
        Ok(())
    }
}
