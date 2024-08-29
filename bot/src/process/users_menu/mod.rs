pub mod list;
pub mod search;

use crate::state::State;
use eyre::Result;
use ledger::Ledger;
use search::{Query, SearchCallback};
use storage::user::User;
use teloxide::{
    types::{CallbackQuery, Message, MessageId},
    Bot,
};
#[derive(Clone, Debug, PartialEq)]
pub enum UserState {
    ShowList((Query, MessageId)),
}

pub async fn go_to_users(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    msg: &Message,
) -> Result<Option<State>> {
    let query = Query::default();
    let msg_id = search::search_users(bot, user, ledger, &query, msg).await?;
    Ok(Some(State::Users(UserState::ShowList((query, msg_id)))))
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
    let data = if let Some(data) = &q.data {
        data
    } else {
        return Ok(Some(State::Users(state)));
    };
    match state {
        UserState::ShowList(query) => match SearchCallback::try_from(data.as_str()) {
            Ok(cmd) => {
                search::handle_callback(bot, user, ledger, query, cmd)
                    .await
            }
            Err(err) => {
                log::warn!("Failed to parse search callback: {:#}", err);
                return Ok(Some(State::Users(UserState::ShowList(query))));
            }
        },
    }
}
