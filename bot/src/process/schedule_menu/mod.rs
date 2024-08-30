use crate::state::State;
use calendar::CalendarState;
use eyre::Result;
use ledger::Ledger;
use storage::user::User;
use teloxide::{
    payloads::SendMessageSetters as _,
    prelude::Requester as _,
    types::{CallbackQuery, Message},
    Bot,
};

use super::Origin;

mod calendar;
mod lending;

#[derive(Clone, Debug)]
pub enum ScheduleState {
    Lending(Origin),
    Calendar(CalendarState),
}

pub async fn go_to_schedule_lending(
    bot: &Bot,
    _: &User,
    _: &Ledger,
    msg: &Message,
) -> Result<Option<State>> {
    let (text, keymap) = lending::render();
    let msg = bot
        .send_message(msg.chat.id, text)
        .reply_markup(keymap)
        .await?;

    ScheduleState::Lending(Origin::from(&msg)).into()
}

pub async fn handle_message(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    message: &Message,
    state: ScheduleState,
) -> Result<Option<State>> {
    match state {
        ScheduleState::Lending(origin) => {
            lending::handle_message(bot, me, ledger, message, origin).await
        }
        ScheduleState::Calendar(state) => {
            calendar::handle_message(bot, me, ledger, message, state).await
        }
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
        ScheduleState::Lending(origin) => {
            lending::handle_callback(bot, me, ledger, q, origin).await
        }
        ScheduleState::Calendar(state) => {
            calendar::handle_callback(bot, me, ledger, q, state).await
        }
    }
}
