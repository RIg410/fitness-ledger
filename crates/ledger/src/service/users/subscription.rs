use super::Users;
use chrono::Utc;
use eyre::{eyre, Result};
use log::info;
use model::session::Session;
use mongodb::bson::oid::ObjectId;
use tx_macro::tx;

impl Users {
    #[tx]
    pub async fn extend_subscriptions(&self, session: &mut Session, days: u32) -> Result<()> {
        info!("Extending subscriptions");
        let mut cursor = self.store.find_all(session, None, None).await?;
        while let Some(user) = cursor.next(session).await {
            let mut user = user?;

            let mut payer = if let Ok(payer) = user.payer_mut() {
                payer
            } else {
                continue;
            };

            for sub in payer.subscriptions_mut() {
                if let model::subscription::Status::Active {
                    start_date: _,
                    end_date,
                } = &mut sub.status
                {
                    *end_date += chrono::Duration::days(days as i64);
                }
            }
            self.store.update(session, &mut payer).await?;
        }
        Ok(())
    }

    #[tx]
    pub async fn change_subscription_balance(
        &self,
        session: &mut Session,
        user_id: ObjectId,
        id: ObjectId,
        delta: i64,
    ) -> Result<()> {
        info!("Changing subscription balance for user {}", user_id);
        let mut user = self
            .store
            .get(session, user_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.resolve_family(session, &mut user).await?;
        let mut payer = user.payer_mut()?;

        payer
            .subscriptions_mut()
            .iter_mut()
            .find(|sub| sub.id == id)
            .map(|sub| {
                if delta > 0 {
                    sub.balance += delta as u32;
                } else {
                    sub.balance = sub.balance.saturating_sub(delta.unsigned_abs() as u32);
                }
            });
        self.store.update(session, &mut payer).await?;
        Ok(())
    }

    #[tx]
    pub async fn change_subscription_locked_balance(
        &self,
        session: &mut Session,
        user_id: ObjectId,
        id: ObjectId,
        delta: i64,
    ) -> Result<()> {
        info!("Changing subscription balance for user {}", user_id);
        let mut user = self
            .store
            .get(session, user_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.resolve_family(session, &mut user).await?;
        let mut payer = user.payer_mut()?;

        payer
            .subscriptions_mut()
            .iter_mut()
            .find(|sub| sub.id == id)
            .map(|sub| {
                if delta > 0 {
                    sub.locked_balance += delta as u32;
                } else {
                    sub.locked_balance = sub.locked_balance.saturating_sub(delta.unsigned_abs() as u32);
                }
            });
        self.store.update(session, &mut payer).await?;
        Ok(())
    }

    #[tx]
    pub async fn change_subscription_days(
        &self,
        session: &mut Session,
        user_id: ObjectId,
        id: ObjectId,
        delta: i64,
    ) -> Result<()> {
        info!("Changing subscription days for user {}", user_id);
        let mut user = self
            .store
            .get(session, user_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.resolve_family(session, &mut user).await?;
        let mut payer = user.payer_mut()?;

        payer
            .subscriptions_mut()
            .iter_mut()
            .find(|sub| sub.id == id)
            .map(|sub| {
                if let model::subscription::Status::Active {
                    start_date: _,
                    end_date,
                } = &mut sub.status
                {
                    if delta > 0 {
                        *end_date += chrono::Duration::days(delta);
                    } else {
                        *end_date -= chrono::Duration::days(delta.abs());
                    }
                }
            });
        self.store.update(session, &mut payer).await?;
        Ok(())
    }

    #[tx]
    pub async fn expire_subscription(&self, session: &mut Session, id: ObjectId) -> Result<bool> {
        let mut user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        let mut payer = if let Ok(payer) = user.payer_mut() {
            payer
        } else {
            log::warn!("User {:?} has no payer", user);
            return Err(eyre!("User has no payer"));
        };

        let now = Utc::now();

        let expired = payer.expire(now);

        let mut expired_sub = false;
        for subscription in expired {
            if !subscription.is_empty() {
                expired_sub = true;
            }

            self.logs
                .expire_subscription(session, id, subscription)
                .await?;
        }
        self.store.update(session, &mut payer).await?;
        Ok(expired_sub)
    }
}
