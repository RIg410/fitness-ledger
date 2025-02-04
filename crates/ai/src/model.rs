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
    pub system_prompt: Option<String>,
}

impl Context {
    pub fn with_system_prompt(system_prompt: String) -> Self {
        Context {
            system_prompt: Some(system_prompt),
        }
    }
}

impl From<Context> for Vec<HistoryEntry> {
    fn from(ctx: Context) -> Self {
        ctx.system_prompt
            .map(|prompt| {
                vec![HistoryEntry {
                    role: Role::System,
                    content: prompt,
                }]
            })
            .unwrap_or_default()
    }
}

pub struct Response {
    pub response: String,
    pub used_words_count: u32,
}
