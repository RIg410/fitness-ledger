use std::fmt::Debug;

use eyre::Result;
use ledger::Ledger;
use storage::user::User;
use teloxide::{
    types::{ChatId, Message, MessageId},
    Bot,
};

use crate::state::State;

pub mod greeting;
pub mod main_menu;
pub mod profile_menu;
pub mod schedule_menu;
pub mod users_menu;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Origin {
    pub chat_id: ChatId,
    pub message_id: MessageId,
}

impl From<&Message> for Origin {
    fn from(msg: &Message) -> Self {
        Self {
            chat_id: msg.chat.id,
            message_id: msg.id,
        }
    }
}

pub async fn proc(
    bot: Bot,
    msg: Message,
    ledger: Ledger,
    state: State,
    mut me: User,
) -> Result<Option<State>> {
    if me.chat_id != msg.chat.id.0 {
        ledger.update_chat_id(&me.user_id, msg.chat.id.0).await?;
        me = ledger
            .get_user_by_tg_id(&me.user_id)
            .await?
            .expect("User not found after update");
    }

    if let Some(text) = msg.text() {
        if text == "/start" {
            main_menu::show_commands(&bot, &me).await?;
            return Ok(None);
        }
    }

    if let Some(state) = main_menu::handle_message(&bot, &me, &ledger, &msg).await? {
        return Ok(Some(state));
    }

    match state {
        State::Start | State::Greeting(_) => {
            main_menu::handle_message(&bot, &me, &ledger, &msg).await
        }
        State::Profile(state) => {
            profile_menu::handle_message(&bot, &me, &ledger, &msg, state).await
        }
        State::Users(state) => users_menu::handle_message(&bot, &me, &ledger, &msg, state).await,
        State::Schedule(state) => {
            schedule_menu::handle_message(&bot, &me, &ledger, &msg, state).await
        }
    }
}
