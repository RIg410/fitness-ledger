pub mod search;
pub mod user_profile;

use crate::state::State;
use eyre::Result;
use ledger::Ledger;
use search::{Query, SearchCallback};
use storage::user::User;
use teloxide::{
    dispatching::dialogue::GetChatId,
    types::{CallbackQuery, ChatId, Message, MessageId},
    Bot,
};
use user_profile::UserCallback;
#[derive(Clone, Debug, PartialEq)]
pub enum UserState {
    ShowList((Query, MessageId)),
    SelectUser((Query, MessageId, String)),
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
    message: &Message,
    state: UserState,
) -> Result<Option<State>> {
    match state {
        UserState::ShowList(query) => {
            search::handle_message(bot, user, ledger, message, query).await
        }
        UserState::SelectUser((query, message_id, user_id)) => {
            user_profile::handle_message(bot, user, ledger, message, (query, message_id, user_id))
                .await
        }
    }
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

    let chat_id = q.chat_id().unwrap_or(ChatId(user.chat_id));
    match state {
        UserState::ShowList(query) => match SearchCallback::try_from(data.as_str()) {
            Ok(cmd) => search::handle_callback(bot, user, ledger, query, cmd, chat_id).await,
            Err(err) => {
                log::warn!("Failed to parse search callback: {:#}", err);
                return Ok(Some(State::Users(UserState::ShowList(query))));
            }
        },
        UserState::SelectUser((query, message_id, user_id)) => {
            match UserCallback::try_from(data.as_str()) {
                Ok(cmd) => {
                    user_profile::handle_callback(
                        bot,
                        user,
                        ledger,
                        (query, message_id, user_id),
                        cmd,
                        chat_id,
                    )
                    .await
                }
                Err(err) => {
                    log::warn!("Failed to parse search callback: {:#}", err);
                    return Ok(Some(State::Users(UserState::SelectUser((
                        query, message_id, user_id,
                    )))));
                }
            }
        }
    }
}
