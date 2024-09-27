use std::sync::{atomic::AtomicBool, Arc};

use eyre::{Context as _, Error};
use ledger::Ledger;
use model::{
    rights::Rule,
    session::Session,
    user::{User, UserIdent},
};
use teloxide::{
    payloads::{EditMessageTextSetters as _, SendMessageSetters as _},
    prelude::Requester,
    types::{ChatId, InlineKeyboardMarkup, InputFile, MessageId, ReplyMarkup},
    ApiError, Bot, RequestError,
};

use crate::sys_button;

pub struct Context {
    pub(crate) bot: Bot,
    pub me: User,
    pub ledger: Ledger,
    origin: Origin,
    pub session: Session,
    pub(crate) system_go_back: bool,
    pub is_real_user: bool,
}

impl Context {
    pub fn new(
        bot: Bot,
        me: User,
        ledger: Ledger,
        origin: Origin,
        session: Session,
        is_real_user: bool,
    ) -> Context {
        Context {
            bot,
            me,
            ledger,
            origin,
            session,
            system_go_back: false,
            is_real_user,
        }
    }

    pub async fn send_document(&mut self, data: Vec<u8>, name: &'static str) -> Result<(), Error> {
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

    pub fn is_me<ID: Into<UserIdent>>(&self, id: ID) -> bool {
        match id.into() {
            UserIdent::TgId(tg_id) => self.me.tg_id == tg_id,
            UserIdent::Id(id) => self.me.id == id,
        }
    }

    pub fn is_couch(&self) -> bool {
        self.me.couch.is_some()
    }

    pub fn is_active(&self) -> bool {
        self.me.is_active
    }

    pub fn chat_id(&self) -> ChatId {
        self.origin.chat_id
    }

    pub(crate) fn origin(&self) -> Origin {
        self.origin.clone()
    }

    pub fn is_admin(&self) -> bool {
        self.me.rights.is_admin()
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

    pub async fn delete_msg(&mut self, id: MessageId) -> Result<(), eyre::Error> {
        if self.origin.message_id == id {
            self.origin.invalidate();
        }
        self.bot.delete_message(self.chat_id(), id).await?;
        Ok(())
    }

    pub fn has_right(&self, rule: Rule) -> bool {
        self.me.rights.has_rule(rule)
    }

    pub fn ensure(&self, rule: Rule) -> Result<(), eyre::Error> {
        self.me.rights.ensure(rule)
    }

    pub async fn send_msg(&mut self, text: &str) -> Result<MessageId, eyre::Error> {
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
        &mut self,
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
        &mut self,
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
}

#[derive(Clone, Debug)]
pub struct Origin {
    pub chat_id: ChatId,
    pub message_id: MessageId,
    pub is_valid: Arc<AtomicBool>,
}

impl Origin {
    pub fn is_valid(&self) -> bool {
        self.is_valid.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn invalidate(&self) {
        self.is_valid
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn set_valid(&self) {
        self.is_valid
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}
