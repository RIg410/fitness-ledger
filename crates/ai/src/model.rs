use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum Model {
    Gpt4oMini,
    Gpt4o,
    Claude3Haiku,
    Claude3Opus,
    Claude3Sonnet,
}

impl Model {
    pub fn name(&self) -> &str {
        match self {
            Model::Gpt4oMini => "gpt-4o-mini",
            Model::Gpt4o => "gpt-4o",
            Model::Claude3Haiku => "claude-3-haiku",
            Model::Claude3Opus => "claude-3-opus",
            Model::Claude3Sonnet => "claude-3.5-sonnet",
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Serialize, Debug)]
pub(crate) struct HistoryEntry {
    pub role: Role,
    pub content: String,
}

#[derive(Serialize)]
pub(crate) struct RequestPayload {
    pub(crate) message: String,
    pub(crate) api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) history: Option<Vec<HistoryEntry>>,
}

impl Debug for RequestPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestPayload")
            .field("message", &self.message)
            .field("history", &self.history)
            .finish()
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct ResponsePayload {
    pub(crate) is_success: bool,
    pub(crate) response: Option<String>,
    pub(crate) used_words_count: Option<u32>,
    pub(crate) error_message: Option<String>,
}

#[derive(Default)]
pub struct Context {
    pub(crate) history: Vec<HistoryEntry>,
}

impl Context {
    pub fn add_system_message(&mut self, message: String) {
        self.history.push(HistoryEntry {
            role: Role::System,
            content: message,
        });
    }

    pub fn add_user_message(&mut self, message: String) {
        self.history.push(HistoryEntry {
            role: Role::User,
            content: message,
        });
    }

    pub fn add_assistant_message(&mut self, message: String) {
        self.history.push(HistoryEntry {
            role: Role::Assistant,
            content: message,
        });
    }
}

impl From<Context> for Vec<HistoryEntry> {
    fn from(ctx: Context) -> Self {
        ctx.history
    }
}

pub struct Response {
    pub response: String,
    pub used_words_count: u32,
}

impl Response {
    pub fn fill_context(self, ctx: &mut Context) {
        ctx.add_assistant_message(self.response);
    }
}
