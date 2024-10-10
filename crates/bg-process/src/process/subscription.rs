use std::sync::Arc;

use crate::{Ledger, Task};
use async_trait::async_trait;
use bot_core::bot::TgBot;
use bot_viewer::fmt_phone;
use eyre::{Error, Result};
use model::rights::Rule;
use teloxide::types::ChatId;

#[derive(Clone)]
pub struct SubscriptionBg {
    ledger: Ledger,
    bot: Arc<TgBot>,
}

#[async_trait]
impl Task for SubscriptionBg {
    const NAME: &'static str = "subscription";
    const CRON: &'static str = "every 2 minutes";

    async fn process(&mut self) -> Result<(), Error> {
        let mut session = self.ledger.db.start_session().await?;

        let users = self
            .ledger
            .users
            .find_subscription_to_expire(&mut session)
            .await?;

        let mut to_notify = vec![];
        for user in users {
            let expired = self
                .ledger
                .users
                .expire_subscription(&mut session, user.id)
                .await?;
            if expired {
                to_notify.push((user.name.first_name, user.phone));
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

        for (name, phone) in to_notify {
            for user in notification_listener.iter() {
                self.bot
                    .send_notification_to(
                        ChatId(user.tg_id),
                        &format!(
                            "У пользователя {}\\({}\\) закончился абонемент",
                            name,
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
    pub fn new(ledger: Ledger, bot: Arc<TgBot>) -> SubscriptionBg {
        SubscriptionBg { ledger, bot }
    }
}
