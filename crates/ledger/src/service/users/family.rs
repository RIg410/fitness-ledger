use eyre::{bail, eyre, Result};
use model::session::Session;
use mongodb::bson::oid::ObjectId;
use tx_macro::tx;

use super::Users;

impl Users {
    #[tx]
    pub async fn remove_family_member(
        &self,
        session: &mut Session,
        user_id: ObjectId,
        member_id: ObjectId,
    ) -> Result<()> {
        let mut user = self
            .store
            .get(session, user_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        let family = &mut user.family;

        let member_idx = family.children_ids.iter().position(|m| *m == member_id);
        if let Some(idx) = member_idx {
            family.children_ids.remove(idx);
        } else {
            bail!("Member not found");
        }
        self.store.update(session, &mut user).await?;

        let mut member = self
            .store
            .get(session, member_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        if member.family.payer_id == Some(user_id) {
            member.family.payer_id = None;
        } else {
            bail!("Member not found");
        }
        self.store.update(session, &mut member).await?;

        self.logs.remove_family_member(session, user_id, member_id).await?;
        Ok(())
    }

    #[tx]
    pub async fn add_family_member(
        &self,
        session: &mut Session,
        user_id: ObjectId,
        member_id: ObjectId,
    ) -> Result<()> {
        let mut user = self
            .store
            .get(session, user_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        let family = &mut user.family;

        if family.children_ids.contains(&member_id) {
            bail!("Member already in family");
        }

        family.children_ids.push(member_id);
        self.store.update(session, &mut user).await?;

        let mut member = self
            .store
            .get(session, member_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        member.family.payer_id = Some(user_id);
        self.store.update(session, &mut member).await?;

        self.logs.add_family_member(session, user_id, member_id).await?;
        Ok(())
    }
}
