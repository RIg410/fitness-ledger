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
mod schedule;

#[derive(Clone, Debug)]
pub enum ScheduleState {
    Lending,
    Schedule,
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

    ScheduleState::Lending.into()
}

pub async fn handle_message(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    message: &Message,
    state: ScheduleState,
) -> Result<Option<State>> {
    match state {
        ScheduleState::Lending => lending::handle_message(bot, me, ledger, message).await,
        ScheduleState::Schedule => schedule::handle_message(bot, me, ledger, message).await,
    }
}

pub async fn handle_callback(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    q: &CallbackQuery,
    state: ScheduleState,
) -> Result<Option<State>> {
    match state {
        ScheduleState::Lending => lending::handle_callback(bot, me, ledger, q).await,
        ScheduleState::Schedule => schedule::handle_callback(bot, me, ledger, q).await,
    }
}
