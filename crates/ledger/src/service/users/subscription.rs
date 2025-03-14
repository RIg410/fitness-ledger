use super::Users;
use chrono::Utc;
use eyre::{eyre, Result};
use log::info;
use model::{decimal::Decimal, session::Session, subscription::UserSubscription};
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
    pub async fn change_subscription_program(
        &self,
        session: &mut Session,
        id: ObjectId,
        subscription_id: ObjectId,
        program_id: ObjectId,
        add: bool,
    ) -> Result<()> {
        let mut user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        self.resolve_family(session, &mut user).await?;
        let mut payer = user.payer_mut()?;

        let sub = payer
            .subscriptions_mut()
            .iter_mut()
            .find(|sub| sub.id == subscription_id);
        if let Some(sub) = sub {
            if let model::subscription::SubscriptionType::Group { program_filter } = &mut sub.tp {
                if add {
                    if program_filter.contains(&program_id) {
                        return Ok(());
                    } else {
                        program_filter.push(program_id);
                    }
                } else {
                    program_filter.retain(|p| p != &program_id);
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
        delta: i32,
    ) -> Result<()> {
        info!("Changing subscription balance for user {}", user_id);
        let mut user = self
            .store
            .get(session, user_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.resolve_family(session, &mut user).await?;
        let mut payer = user.payer_mut()?;

        self.logs.change_balance(session, payer.id, delta).await?;

        payer
            .subscriptions_mut()
            .iter_mut()
            .find(|sub| sub.id == id)
            .map(|sub| {
                if delta > 0 {
                    sub.balance += delta as u32;
                } else {
                    sub.balance = sub.balance.saturating_sub(delta.unsigned_abs());
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
        delta: i32,
    ) -> Result<()> {
        info!("Changing subscription balance for user {}", user_id);
        let mut user = self
            .store
            .get(session, user_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.resolve_family(session, &mut user).await?;
        let mut payer = user.payer_mut()?;

        self.logs
            .change_reserved_balance(session, payer.id, delta)
            .await?;

        payer
            .subscriptions_mut()
            .iter_mut()
            .find(|sub| sub.id == id)
            .map(|sub| {
                if delta > 0 {
                    sub.locked_balance += delta as u32;
                } else {
                    sub.locked_balance = sub
                        .locked_balance
                        .saturating_sub(delta.unsigned_abs());
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
        delta: i32,
    ) -> Result<()> {
        info!("Changing subscription days for user {}", user_id);
        let mut user = self
            .store
            .get(session, user_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.resolve_family(session, &mut user).await?;
        let mut payer = user.payer_mut()?;

        let payer_id = payer.id;
        let sub = payer
            .subscriptions_mut()
            .iter_mut()
            .find(|sub| sub.id == id);
        if let Some(sub) = sub {
            if let model::subscription::Status::Active {
                start_date: _,
                end_date,
            } = &mut sub.status
            {
                self.logs
                    .change_subscription_days(session, payer_id, delta)
                    .await?;
                if delta > 0 {
                    *end_date += chrono::Duration::days(delta as i64);
                } else {
                    *end_date -= chrono::Duration::days(delta.abs() as i64);
                }
            }
        }
        self.store.update(session, &mut payer).await?;
        Ok(())
    }

    #[tx]
    pub async fn expire_subscription(
        &self,
        session: &mut Session,
        id: ObjectId,
    ) -> Result<Vec<UserSubscription>> {
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

        for subscription in &expired {
            self.logs
                .expire_subscription(session, id, subscription.clone())
                .await?;
        }
        self.store.update(session, &mut payer).await?;
        Ok(expired)
    }

    #[tx]
    pub async fn set_subscription_item_price(
        &self,
        session: &mut Session,
        user_id: ObjectId,
        id: ObjectId,
        price: Decimal,
    ) -> Result<()> {
        info!("Setting subscription item price for user {}", id);

        let mut user = self
            .store
            .get(session, user_id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;

        self.resolve_family(session, &mut user).await?;
        let mut payer = user.payer_mut()?;

        if let Some(sub) = payer
            .subscriptions_mut()
            .iter_mut()
            .find(|sub| sub.id == id) { sub.item_price = Some(price); }
        self.store.update(session, &mut payer).await?;
        Ok(())
    }
}
