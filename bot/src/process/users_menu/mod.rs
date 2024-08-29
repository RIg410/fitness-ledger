pub mod search;
pub mod user_profile;

use crate::state::State;
use eyre::Result;
use ledger::Ledger;
use search::{Query, SearchCallback};
use storage::user::{
    rights::{Rule, TrainingRule},
    User,
};
use teloxide::{
    dispatching::dialogue::GetChatId,
    types::{CallbackQuery, ChatId, Message, MessageId},
    Bot,
};
use user_profile::UserCallback;

#[derive(Clone, Debug, PartialEq)]
pub enum UserState {
    ShowList(UserListParams),
    SelectUser(SelectedUser),
    UserRights(SelectedUser),
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserListParams {
    pub query: Query,
    pub message_id: MessageId,
}

impl UserListParams {
    pub fn new(query: Query, message_id: MessageId) -> Self {
        Self { query, message_id }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectedUser {
    pub list: UserListParams,
    pub user_id: String,
}

impl SelectedUser {
    pub fn new(list: UserListParams, user_id: String) -> Self {
        Self { list, user_id }
    }
}

pub async fn go_to_users(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    msg: &Message,
) -> Result<Option<State>> {
    let query = Query::default();
    let msg_id = search::search_users(bot, user, ledger, &query, msg).await?;
    Ok(Some(State::Users(UserState::ShowList(
        UserListParams::new(query, msg_id),
    ))))
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
        UserState::SelectUser(selected_user) => {
            user_profile::handle_message(bot, user, ledger, message, selected_user).await
        }
        UserState::UserRights(_) => todo!(),
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
        UserState::SelectUser(selected_user) => match UserCallback::try_from(data.as_str()) {
            Ok(cmd) => {
                user_profile::handle_callback(bot, user, ledger, selected_user, cmd, chat_id).await
            }
            Err(err) => {
                log::warn!("Failed to parse search callback: {:#}", err);
                return Ok(Some(State::Users(UserState::SelectUser(selected_user))));
            }
        },
        UserState::UserRights(_) => todo!(),
    }
}

pub fn user_type(user: &User) -> &str {
    if !user.is_active {
        "⚫"
    } else if user.rights.has_rule(Rule::Full) {
        "🔴"
    } else if user.rights.has_rule(Rule::Training(TrainingRule::Train)) {
        "🔵"
    } else {
        "🟢"
    }
}
