use std::sync::Arc;

use crate::{Ledger, Task};
use async_trait::async_trait;
use bot_core::bot::TgBot;
use bot_viewer::{fmt_phone, user::tg_link};
use chrono::Utc;
use eyre::{Error, Result};
use model::rights::Rule;
use teloxide::types::ChatId;

#[derive(Clone)]
pub struct SubscriptionBg {
    ledger: Arc<Ledger>,
    bot: Arc<TgBot>,
}

#[async_trait]
impl Task for SubscriptionBg {
    const NAME: &'static str = "subscription";
    const CRON: &'static str = "every day at 7:00";

    async fn process(&mut self) -> Result<(), Error> {
        let mut session = self.ledger.db.start_session().await?;

        let mut users = self
            .ledger
            .users
            .find_users_with_active_subs(&mut session)
            .await?;
        let mut to_notify = vec![];

        while let Some(user) = users.next(&mut session).await {
            let user = user?;
            let expired = user
                .subscriptions
                .iter()
                .any(|sub| sub.is_expired(Utc::now()));
            if expired {
                let expired = self
                    .ledger
                    .users
                    .expire_subscription(&mut session, user.id)
                    .await?;
                if expired {
                    log::info!("User {:?} has expired subscription", user);
                    to_notify.push((user.tg_id, user.name.first_name, user.phone));
                }
            }
        }

        if to_notify.is_empty() {
            return Ok(());
        }

        let notification_listener = self
            .ledger
            .users
            .find_users_with_right(&mut session, Rule::ReceiveNotificationsAboutSubscriptions)
            .await?;
        for (id, name, phone) in to_notify {
            for user in notification_listener.iter() {
                self.bot
                    .send_notification_to(
                        ChatId(user.tg_id),
                        &format!(
                            "У пользователя {}\\({}\\) сгорел абонемент",
                            tg_link(id, Some(name.as_str())),
                            fmt_phone(&phone)
                        ),
                    )
                    .await?;
            }
        }

        Ok(())
    }
}

impl SubscriptionBg {
    pub fn new(ledger: Arc<Ledger>, bot: Arc<TgBot>) -> SubscriptionBg {
        SubscriptionBg { ledger, bot }
    }
}
