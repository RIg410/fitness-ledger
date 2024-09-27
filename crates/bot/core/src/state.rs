use crate::{context::Origin, widget::Widget};
use eyre::{bail, Error};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use teloxide::types::ChatId;

#[derive(Default)]
pub struct State {
    pub view: Option<Widget>,
    pub origin: Option<Origin>,
}

#[derive(Default, Clone)]
pub struct StateHolder {
    map: Arc<Mutex<HashMap<ChatId, State>>>,
}

impl StateHolder {
    pub fn get_state(&self, chat_id: ChatId) -> Option<State> {
        let mut map = self.map.lock().unwrap();
        map.remove(&chat_id)
    }

    pub fn set_state(&self, chat_id: ChatId, state: State) {
        let mut map = self.map.lock().unwrap();
        map.insert(chat_id, state);
    }

    pub fn try_get_origin(&self, chat_id: ChatId) -> Result<Option<Origin>, Error> {
        match self.map.try_lock() {
            Ok(lock) => Ok(lock.get(&chat_id).and_then(|s| s.origin.clone())),
            Err(_) => bail!("locked"),
        }
    }
}
