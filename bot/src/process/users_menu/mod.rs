use crate::state::State;
use eyre::Result;
use ledger::Ledger;
use storage::user::User;
use teloxide::{types::{CallbackQuery, Message}, Bot};
#[derive(Clone, Debug, Default, PartialEq)]
pub enum UserState {
    #[default]
    ShowList,
}

pub async fn go_to_users(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    msg: &Message,
) -> Result<Option<State>> {
    
    Ok(None)
}

pub async fn handle_message(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    msg: &Message,
    state: UserState,
) -> Result<Option<State>> {
    Ok(None)
}

pub async fn handle_callback(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    q: &CallbackQuery,
    state: UserState,
) -> Result<Option<State>> {
    Ok(None)
}