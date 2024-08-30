use eyre::Context as _;
use ledger::Ledger;
use storage::user::{rights::Rule, User};
use teloxide::{
    payloads::SendMessageSetters as _,
    prelude::{Requester, ResponseResult},
    types::{ChatId, KeyboardMarkup, MessageId},
    Bot,
};

pub struct Context {
    bot: Bot,
    me: User,
    pub ledger: Ledger,
    origin: Origin,
}

impl Context {
    pub fn new(bot: Bot, me: User, ledger: Ledger, origin: Origin) -> Context {
        Context {
            bot,
            me,
            ledger,
            origin,
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

    pub fn update_origin(&mut self, id: MessageId) {
        self.origin.message_id = id;
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

    pub async fn send_replay_markup(
        &self,
        text: &str,
        markup: KeyboardMarkup,
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
