use crate::{
    bot::{Origin, ValidToken},
    widget::Widget,
};
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};
use teloxide::types::ChatId;

#[derive(Default)]
pub struct State {
    pub view: Option<Widget>,
    pub origin: Option<Origin>,
}

#[derive(Default, Clone)]
pub struct StateHolder {
    map: Arc<Mutex<HashMap<ChatId, State>>>,
    tokens: Tokens,
}

impl StateHolder {
    pub fn get_state(&self, chat_id: ChatId) -> Option<State> {
        let mut map = self.map.lock();
        map.remove(&chat_id)
    }

    pub fn set_state(&self, chat_id: ChatId, state: State) {
        let mut map = self.map.lock();
        map.insert(chat_id, state);
    }

    pub fn tokens(&self) -> Tokens {
        self.tokens.clone()
    }

    pub fn get_token(&self, chat_id: ChatId) -> ValidToken {
        self.tokens.get_token(chat_id)
    }
}

#[derive(Clone, Default)]
pub struct Tokens {
    tokens: Arc<Mutex<HashMap<ChatId, ValidToken>>>,
}

impl Tokens {
    pub fn new() -> Self {
        Tokens {
            tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_token(&self, chat_id: ChatId) -> ValidToken {
        let mut tokens = self.tokens.lock();
        tokens
            .entry(chat_id)
            .or_insert_with(ValidToken::new)
            .clone()
    }
}
