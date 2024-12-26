use eyre::{bail, eyre, Result};
use model::{
    rights::Rights,
    session::Session,
    user::{extension::UserExtension, User, UserName},
};
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

        self.logs
            .remove_family_member(session, user_id, member_id)
            .await?;
        Ok(())
    }

    #[tx]
    pub async fn create_family_member(
        &self,
        session: &mut Session,
        user_id: ObjectId,
        name: &str,
        surname: &Option<String>,
    ) -> Result<()> {
        let mut user = self
            .store
            .get(session, user_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        let mut child = User::new(
            -1,
            UserName {
                tg_user_name: None,
                first_name: name.to_string(),
                last_name: surname.clone(),
            },
            Rights::customer(),
            None,
            user.come_from.clone(),
        );
        child.family.payer_id = Some(user_id);
        let id = child.id;

        self.store.insert(session, child).await?;

        user.family.children_ids.push(id);
        self.store.update(session, &mut user).await?;

        self.store
            .update_extension(
                session,
                UserExtension {
                    id,
                    birthday: None,
                    notification_mask: Default::default(),
                },
            )
            .await?;

        Ok(())
    }

    #[tx]
    pub async fn add_family_member(
        &self,
        session: &mut Session,
        parent_id: ObjectId,
        member_id: ObjectId,
    ) -> Result<()> {
        let mut parent = self
            .store
            .get(session, parent_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        if parent.family.payer_id.is_some() {
            bail!("User already has family");
        }

        if parent.family.children_ids.contains(&member_id) {
            bail!("Member already in family");
        }

        parent.family.children_ids.push(member_id);
        self.store.update(session, &mut parent).await?;

        let mut member = self
            .store
            .get(session, member_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        if member.has_subscriptions() {
            bail!("Member already has subscriptions");
        }

        if member.family.exists() {
            bail!("Member already in family");
        }

        member.family.payer_id = Some(parent_id);
        self.store.update(session, &mut member).await?;

        self.logs
            .add_family_member(session, parent_id, member_id)
            .await?;
        Ok(())
    }
}
