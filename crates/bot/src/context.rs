use eyre::Context as _;
use ledger::Ledger;
use model::{rights::Rule, user::User};
use mongodb::ClientSession;
use teloxide::{
    payloads::{EditMessageTextSetters as _, SendMessageSetters as _},
    prelude::Requester,
    types::{ChatId, InlineKeyboardMarkup, KeyboardMarkup, MessageId, ReplyMarkup},
    ApiError, Bot, RequestError,
};

pub struct Context {
    pub bot: Bot,
    pub me: User,
    pub ledger: Ledger,
    origin: Origin,
    pub session: ClientSession,
}

impl Context {
    pub fn new(
        bot: Bot,
        me: User,
        ledger: Ledger,
        origin: Origin,
        session: ClientSession,
    ) -> Context {
        Context {
            bot,
            me,
            ledger,
            origin,
            session,
        }
    }

    pub fn is_active(&self) -> bool {
        self.me.is_active
    }

    pub fn chat_id(&self) -> ChatId {
        self.origin.chat_id
    }

    pub fn origin(&self) -> Origin {
        self.origin
    }

    pub fn update_origin_msg_id(&mut self, id: MessageId) {
        self.origin.message_id = id;
    }

    pub async fn edit_origin(
        &self,
        text: &str,
        markup: InlineKeyboardMarkup,
    ) -> Result<(), eyre::Error> {
        let update_result = self
            .bot
            .edit_message_text(self.chat_id(), self.origin.message_id, text)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .reply_markup(markup)
            .await;
        match update_result {
            Ok(_) => Ok(()),
            Err(RequestError::Api(ApiError::MessageNotModified)) => Ok(()),
            Err(e) => {
                log::error!("Failed to edit message: {}: {}", e, text);
                Err(e.into())
            }
        }
    }

    pub async fn delete_msg(&self, id: MessageId) -> Result<(), eyre::Error> {
        self.bot.delete_message(self.chat_id(), id).await?;
        Ok(())
    }

    pub fn has_right(&self, rule: Rule) -> bool {
        self.me.rights.has_rule(rule)
    }

    pub fn ensure(&self, rule: Rule) -> Result<(), eyre::Error> {
        self.me.rights.ensure(rule)
    }

    pub async fn send_msg(&self, text: &str) -> Result<MessageId, eyre::Error> {
        Ok(self
            .bot
            .send_message(self.chat_id(), text)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await
            .context(format!("Failed to send message: {}", text))?
            .id)
    }

    pub async fn send_msg_with_markup(
        &self,
        text: &str,
        markup: InlineKeyboardMarkup,
    ) -> Result<MessageId, eyre::Error> {
        Ok(self
            .bot
            .send_message(self.chat_id(), text)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .reply_markup(markup)
            .await
            .context(format!("Failed to send message: {}", text))?
            .id)
    }

    pub async fn reload_user(&mut self) -> Result<(), eyre::Error> {
        let user = self
            .ledger
            .users
            .get_by_tg_id(&mut self.session, self.me.tg_id)
            .await?
            .ok_or_else(|| eyre::eyre!("Failed to load existing user:{}", self.me.id))?;

        self.me = user;
        Ok(())
    }

    pub async fn send_replay_markup(
        &self,
        text: &str,
        markup: ReplyMarkup,
    ) -> Result<MessageId, eyre::Error> {
        Ok(self
            .bot
            .send_message(self.chat_id(), text)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .reply_markup(markup)
            .await
            .context(format!("Failed to send message: {}", text))?
            .id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Origin {
    pub chat_id: ChatId,
    pub message_id: MessageId,
}
