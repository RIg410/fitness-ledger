use crate::state::State;
use eyre::Result;
use ledger::Ledger;
use storage::user::User;
use teloxide::{
    payloads::SendMessageSetters as _,
    prelude::Requester as _,
    types::{CallbackQuery, Message},
    Bot,
};

mod lending;

#[derive(Clone, Debug)]
pub enum ScheduleState {
    Start,
}

pub async fn go_to_schedule(
    bot: &Bot,
    _: &User,
    _: &Ledger,
    msg: &Message,
) -> Result<Option<State>> {
    let (text, keymap) = lending::render();
    bot.send_message(msg.chat.id, text)
        .reply_markup(keymap)
        .await?;

    ScheduleState::Start.into()
}

pub async fn handle_message(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    message: &Message,
    state: ScheduleState,
) -> Result<Option<State>> {
    println!("handle_message");
    todo!()
}

pub async fn handle_callback(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    q: &CallbackQuery,
    state: ScheduleState,
) -> Result<Option<State>> {
    println!("handle_callback");
    todo!()
}
