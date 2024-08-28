use eyre::Result;
use ledger::Ledger;
use storage::user::User;
use teloxide::{types::Message, Bot};

use crate::state::State;

pub mod greeting;
pub mod main_menu;
pub mod profile_menu;

pub async fn proc(
    bot: Bot,
    msg: Message,
    ledger: Ledger,
    state: State,
    user: User,
) -> Result<Option<State>> {
    if let Some(text) = msg.text() {
        if text == "/start" {
            main_menu::show_commands(&bot, &user).await?;
            return Ok(None);
        }
    }
    
    if let Some(state) = main_menu::handle_message(&bot, &user, &ledger, &msg).await? {
        return Ok(Some(state));
    }

    match state {
        State::Start | State::Greeting(_) => {
            main_menu::handle_message(&bot, &user, &ledger, &msg).await
        }
        State::Profile(state) => {
            profile_menu::handle_message(&bot, &user, &ledger, &msg, state).await
        }
    }
}
