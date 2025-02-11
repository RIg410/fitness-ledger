use crate::Task;
use async_trait::async_trait;
use bot_core::{bot::TgBot, CommonLocation};
use bot_viewer::fmt_phone;
use chrono::{Datelike as _, Local};
use eyre::Error;
use ledger::Ledger;
use log::info;
use model::{rights::Rule, user::User};
use std::sync::Arc;
use teloxide::{
    types::{ChatId, InlineKeyboardMarkup},
    utils::markdown::escape,
};

#[derive(Clone)]
pub struct MotivationNotifier {
    pub ledger: Arc<Ledger>,
    pub bot: Arc<TgBot>,
}

impl MotivationNotifier {
    pub fn new(ledger: Arc<Ledger>, bot: Arc<TgBot>) -> MotivationNotifier {
        MotivationNotifier { ledger, bot }
    }
}

#[async_trait]
impl Task for MotivationNotifier {
    const NAME: &'static str = "motivation-notifier";
    const CRON: &'static str = "every 1 day at 12:12";

    async fn process(&mut self) -> Result<(), Error> {
        let mut session = self.ledger.db.start_session().await?;

        let notification_listener = self
            .ledger
            .users
            .find_users_with_right(&mut session, Rule::ReceiveAiNotifications)
            .await?;

        for user in notification_listener {
            let extension = self
                .ledger
                .users
                .get_extension(&mut session, user.id)
                .await?;
            if let Some(prompt) = extension.ai_message_prompt {
                if let Ok(response) = self
                    .ledger
                    .ai
                    .ask(ai::AiModel::Gpt4oMini, prompt, None)
                    .await
                {
                    self.bot
                        .notify(ChatId(user.tg_id), &format!("{}", response.response), false)
                        .await;
                }
            }
        }

        Ok(())
    }
}
