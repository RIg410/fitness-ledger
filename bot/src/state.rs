use crate::process::{greeting::Greeting, profile_menu::ProfileState, schedule_menu::ScheduleState, users_menu::UserState};
use eyre::Result;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use teloxide::types::ChatId;

#[derive(Clone, Debug, Default)]
pub enum State {
    #[default]
    Start,
    Greeting(Greeting),
    Profile(ProfileState),
    Users(UserState),
    Schedule(ScheduleState),
}

impl From<UserState> for Result<Option<State>> {
    fn from(state: UserState) -> Self {
        Ok(Some(State::Users(state)))
    }
}

impl From<Greeting> for Result<Option<State>> {
    fn from(greeting: Greeting) -> Self {
        Ok(Some(State::Greeting(greeting)))
    }
}

impl From<ProfileState> for Result<Option<State>> {
    fn from(state: ProfileState) -> Self {
        Ok(Some(State::Profile(state)))
    }
}

impl From<ScheduleState> for Result<Option<State>> {
    fn from(state: ScheduleState) -> Self {
        Ok(Some(State::Schedule(state)))
    }
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
