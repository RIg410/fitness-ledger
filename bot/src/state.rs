use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use teloxide::types::ChatId;

use crate::process::{greeting::Greeting, profile_menu::ProfileState};

#[derive(Clone, Debug, Default)]
pub enum State {
    #[default]
    Start,
    Greeting(Greeting),
    Profile(ProfileState),
}

#[derive(Default, Clone)]
pub struct StateHolder {
    map: Arc<Mutex<HashMap<ChatId, State>>>,
}

impl StateHolder {
    pub fn get_state(&self, chat_id: ChatId) -> Option<State> {
        let map = self.map.lock().unwrap();
        map.get(&chat_id).cloned()
    }

    pub fn set_state(&self, chat_id: ChatId, state: State) {
        let mut map = self.map.lock().unwrap();
        map.insert(chat_id, state);
    }

    pub fn remove_state(&self, chat_id: ChatId) {
        let mut map = self.map.lock().unwrap();
        map.remove(&chat_id);
    }
}
