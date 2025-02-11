use std::sync::Arc;

use crate::{Ledger, Task};
use async_trait::async_trait;
use bot_core::{bot::TgBot, CommonLocation};
use bot_viewer::{fmt_phone, user::tg_link};
use chrono::Utc;
use eyre::{Error, Result};
use model::rights::Rule;
use teloxide::{
    types::{ChatId, InlineKeyboardMarkup},
    utils::markdown::escape,
};

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

        let notification_listener = self
            .ledger
            .users
            .find_users_with_right(&mut session, Rule::ReceiveNotificationsAboutSubscriptions)
            .await?;

        while let Some(user) = users.next(&mut session).await {
            let user = user?;
            let extension = self
                .ledger
                .users
                .get_extension(&mut session, user.id)
                .await?;

            let payer = if let Ok(payer) = user.payer() {
                payer
            } else {
                log::warn!("User {:?} has no payer", user);
                continue;
            };

            let expired = payer
                .subscriptions()
                .iter()
                .any(|sub| sub.is_expired(Utc::now()));
            if expired {
                let expired = self
                    .ledger
                    .users
                    .expire_subscription(&mut session, user.id)
                    .await?;
                for sub in expired {
                    for listener in notification_listener.iter() {
                        self.bot
                            .notify_with_markup(
                                ChatId(listener.tg_id),
                                &format!(
                                    "У пользователя {}\\({}\\) сгорел абонемент: {}\\. Сгорело {}\\.",
                                    tg_link(user.tg_id, user.name.tg_user_name.as_deref()),
                                    fmt_phone(user.phone.as_deref()),
                                    escape(sub.name.as_str()),
                                    sub.balance,
                                ),
                                InlineKeyboardMarkup::default().append_row(vec![CommonLocation::Profile(user.id).button()]),
                            )
                            .await;

                        if extension.birthday.is_none() {
                            self.bot
                                .notify(ChatId(listener.tg_id), "У пользователя нет даты рождения", true)
                                .await;
                        }
                    }
                }
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
