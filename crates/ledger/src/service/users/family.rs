use eyre::Result;
use model::{
    errors::LedgerError,
    rights::Rights,
    session::Session,
    user::{User, UserName},
};
use mongodb::bson::oid::ObjectId;
use tx_macro::tx;

use super::Users;

impl Users {
    #[tx]
    pub async fn set_individual_family_member(
        &self,
        session: &mut Session,
        member_id: ObjectId,
        is_individual: bool,
    ) -> Result<(), LedgerError> {
        let mut user = self
            .store
            .get(session, member_id)
            .await?
            .ok_or_else(|| LedgerError::UserNotFound(member_id))?;
        user.family.is_individual = is_individual;
        self.store.update(session, &mut user).await?;
        Ok(())
    }

    #[tx]
    pub async fn remove_family_member(
        &self,
        session: &mut Session,
        user_id: ObjectId,
        member_id: ObjectId,
    ) -> Result<(), LedgerError> {
        let mut user = self
            .store
            .get(session, user_id)
            .await?
            .ok_or_else(|| LedgerError::UserNotFound(user_id))?;

        let family = &mut user.family;

        let member_idx = family.children_ids.iter().position(|m| *m == member_id);
        if let Some(idx) = member_idx {
            family.children_ids.remove(idx);
        } else {
            return Err(LedgerError::MemberNotFound { user_id, member_id });
        }
        self.store.update(session, &mut user).await?;

        let mut member = self
            .store
            .get(session, member_id)
            .await?
            .ok_or_else(|| LedgerError::UserNotFound(member_id))?;

        if member.family.payer_id == Some(user_id) {
            member.family.payer_id = None;
        } else {
            return Err(LedgerError::WrongFamilyMember { user_id, member_id });
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
    ) -> Result<(), LedgerError> {
        let mut user = self
            .store
            .get(session, user_id)
            .await?
            .ok_or_else(|| LedgerError::UserNotFound(user_id))?;

        let mut child = User::new(
            -1,
            UserName {
                tg_user_name: None,
                first_name: name.to_string(),
                last_name: surname.clone(),
            },
            Rights::customer(),
            None,
            user.come_from,
        );
        child.family.payer_id = Some(user_id);
        let id = child.id;

        self.logs
            .add_family_member(session, user_id, child.id)
            .await?;
        self.store.insert(session, child).await?;

        user.family.children_ids.push(id);
        self.store.update(session, &mut user).await?;

        Ok(())
    }

    #[tx]
    pub async fn add_family_member(
        &self,
        session: &mut Session,
        parent_id: ObjectId,
        member_id: ObjectId,
    ) -> Result<(), LedgerError> {
        let mut parent = self
            .store
            .get(session, parent_id)
            .await?
            .ok_or_else(|| LedgerError::UserNotFound(parent_id))?;

        if let Some(payer_id) = parent.family.payer_id {
            parent = self
                .store
                .get(session, payer_id)
                .await?
                .ok_or_else(|| LedgerError::UserNotFound(payer_id))?;
        }

        if parent.family.children_ids.contains(&member_id) {
            return Err(LedgerError::UserAlreadyInFamily {
                user_id: parent.id,
                member_id,
            });
        }

        parent.family.children_ids.push(member_id);
        self.store.update(session, &mut parent).await?;

        let mut member = self
            .store
            .get(session, member_id)
            .await?
            .ok_or_else(|| LedgerError::UserNotFound(member_id))?;

        if member.family.exists() {
            return Err(LedgerError::UserAlreadyInFamily {
                user_id: parent.id,
                member_id,
            });
        }

        member.family.payer_id = Some(parent_id);
        self.store.update(session, &mut member).await?;

        self.logs
            .add_family_member(session, parent_id, member_id)
            .await?;
        Ok(())
    }
}
