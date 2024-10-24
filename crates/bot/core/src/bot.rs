use crate::{state::Tokens, sys_button};
use eyre::{Context as _, Error};
use std::{
    fmt::Debug,
    sync::{atomic::AtomicBool, Arc},
};
use teloxide::{
    payloads::{EditMessageTextSetters as _, SendMessageSetters as _},
    prelude::Requester as _,
    types::{ChatId, InlineKeyboardMarkup, InputFile, MessageId, ParseMode, ReplyMarkup, True},
    ApiError, Bot, RequestError,
};

pub struct TgBot {
    bot: Bot,
    tokens: Tokens,
    origin: Origin,
    system_go_back: bool,
}

impl TgBot {
    pub fn new(bot: Bot, tokens: Tokens, origin: Origin) -> Self {
        TgBot {
            bot,
            tokens,
            origin,
            system_go_back: false,
        }
    }

    pub async fn send_document(&self, data: Vec<u8>, name: &'static str) -> Result<(), Error> {
        self.origin.invalidate();
        self.bot
            .send_document(self.chat_id(), InputFile::memory(data).file_name(name))
            .await?;
        Ok(())
    }

    pub async fn send_notification(&mut self, err: &str) -> Result<(), Error> {
        self.send_msg(err).await?;
        self.reset_origin().await?;
        Ok(())
    }

    pub async fn reset_origin(&mut self) -> Result<(), Error> {
        let id = self.send_msg("\\.").await?;
        self.origin.message_id = id;
        self.origin.set_valid();
        Ok(())
    }

    pub async fn edit_origin(
        &mut self,
        text: &str,
        markup: InlineKeyboardMarkup,
    ) -> Result<(), eyre::Error> {
        if !self.origin.is_valid() {
            self.reset_origin().await?;
        }

        let update_result = self
            .bot
            .edit_message_text(self.chat_id(), self.origin.message_id, text)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .reply_markup(sys_button(markup, self.system_go_back))
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
        if self.origin.message_id == id {
            self.origin.invalidate();
        }
        self.bot.delete_message(self.chat_id(), id).await?;
        Ok(())
    }

    pub async fn send_msg(&self, text: &str) -> Result<MessageId, eyre::Error> {
        self.origin.invalidate();

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
        self.origin.invalidate();

        Ok(self
            .bot
            .send_message(self.chat_id(), text)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .reply_markup(markup)
            .await
            .context(format!("Failed to send message: {}", text))?
            .id)
    }

    pub async fn send_replay_markup(
        &self,
        text: &str,
        markup: ReplyMarkup,
    ) -> Result<MessageId, eyre::Error> {
        self.origin.invalidate();

        Ok(self
            .bot
            .send_message(self.chat_id(), text)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .reply_markup(markup)
            .await
            .context(format!("Failed to send message: {}", text))?
            .id)
    }

    pub fn chat_id(&self) -> ChatId {
        self.origin.chat_id
    }

    pub fn set_system_go_back(&mut self, system_go_back: bool) {
        self.system_go_back = system_go_back;
    }

    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    pub async fn answer_callback_query<C: Into<String>>(
        &self,
        id: C,
    ) -> Result<True, RequestError> {
        self.bot.answer_callback_query(id).await
    }

    pub async fn pin_message(&self, chat_id: ChatId, id: MessageId) -> Result<(), RequestError> {
        if id.0 == 0 {
            return Ok(());
        }

        self.bot.pin_chat_message(chat_id, id).await?;
        Ok(())
    }

    pub async fn send_notification_to(
        &self,
        chat_id: ChatId,
        text: &str,
    ) -> Result<MessageId, RequestError> {
        if chat_id.0 == -1 {
            return Ok(MessageId(0));
        }
        let id = self
            .bot
            .send_message(chat_id, text)
            .parse_mode(ParseMode::MarkdownV2)
            .await?
            .id;
        let tkn = self.tokens.get_token(chat_id);
        tkn.invalidate();
        Ok(id)
    }
}

#[derive(Clone, Debug)]
pub struct Origin {
    pub chat_id: ChatId,
    pub message_id: MessageId,
    pub tkn: ValidToken,
}

impl Origin {
    pub fn is_valid(&self) -> bool {
        self.tkn.is_valid()
    }

    pub fn invalidate(&self) {
        self.tkn.invalidate();
    }

    pub fn set_valid(&self) {
        self.tkn.set_valid();
    }
}

#[derive(Clone)]
pub struct ValidToken(Arc<AtomicBool>);

impl ValidToken {
    pub fn new() -> Self {
        ValidToken(Arc::new(AtomicBool::new(true)))
    }

    pub fn is_valid(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn invalidate(&self) {
        self.0.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn set_valid(&self) {
        self.0.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Debug for ValidToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidToken")
            .field("is_valid", &self.is_valid())
            .finish()
    }
}

impl Default for ValidToken {
    fn default() -> Self {
        Self::new()
    }
}
