use eyre::{eyre, Result};
use ledger::Ledger;
use storage::user::{rights::Rule, User};
use teloxide::{
    payloads::{EditMessageTextSetters as _, SendMessageSetters as _},
    prelude::Requester as _,
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageId},
    utils::markdown::escape,
    Bot,
};

use crate::{process::users_menu::UserState, state::State};

use super::{user_profile::show_user_profile, user_type, SelectedUser, UserListParams};

pub const LIMIT: u64 = 7;

#[derive(Clone, Debug, PartialEq)]
pub enum SearchCallback {
    Next,
    Prev,
    Select(String),
}

impl TryFrom<&str> for SearchCallback {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "sc_next" {
            Ok(SearchCallback::Next)
        } else if value == "sc_prev" {
            Ok(SearchCallback::Prev)
        } else if value.starts_with("sc_select:") {
            Ok(SearchCallback::Select(value[10..].to_string()))
        } else {
            Err(eyre!("Invalid search callback:{}", value))
        }
    }
}

impl SearchCallback {
    pub fn to_data(&self) -> String {
        match self {
            SearchCallback::Next => "sc_next".to_string(),
            SearchCallback::Prev => "sc_prev".to_string(),
            SearchCallback::Select(user_id) => format!("sc_select:{}", user_id),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    pub query: String,
    pub offset: u64,
}

impl Default for Query {
    fn default() -> Self {
        Query {
            query: "".to_string(),
            offset: 0,
        }
    }
}

pub async fn handle_message(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    msg: &Message,
    state: UserListParams,
) -> Result<Option<State>> {
    bot.delete_message(msg.chat.id, msg.id).await?;
    me.rights.ensure(Rule::ViewUsers)?;

    let mut query = msg.text().to_owned().unwrap_or_default().to_string();
    if query.len() == 1 && !query.chars().next().unwrap().is_alphanumeric() {
        query = "".to_string();
    }

    let query = Query { query, offset: 0 };
    let params = UserListParams::new(query, state.message_id);

    update_search(bot, me, ledger, msg.chat.id, &params).await?;
    Ok(Some(State::Users(UserState::ShowList(params))))
}

pub async fn handle_callback(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    list_params: UserListParams,
    cmd: SearchCallback,
    chat_id: ChatId,
) -> Result<Option<State>> {
    me.rights.ensure(Rule::ViewUsers)?;

    match cmd {
        SearchCallback::Next => {
            let new_query = Query {
                query: list_params.query.query.clone(),
                offset: list_params.query.offset + LIMIT,
            };
            let params = UserListParams::new(new_query, list_params.message_id);
            update_search(bot, me, ledger, chat_id, &params).await?;
            UserState::ShowList(params).into()
        }
        SearchCallback::Prev => {
            let new_query = Query {
                query: list_params.query.query.clone(),
                offset: list_params.query.offset.saturating_sub(LIMIT),
            };
            let params = UserListParams::new(new_query, list_params.message_id);
            update_search(bot, me, ledger, chat_id, &params).await?;
            UserState::ShowList(params).into()
        }
        SearchCallback::Select(user_id) => {
            show_user_profile(
                bot,
                me,
                ledger,
                user_id.clone(),
                chat_id,
                list_params.message_id,
            )
            .await?;
            UserState::SelectUser(SelectedUser::new(list_params.clone(), user_id)).into()
        }
    }
}

pub async fn search_users(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    query: &Query,
    msg: &Message,
) -> Result<MessageId> {
    me.rights.ensure(Rule::ViewUsers)?;

    let count = ledger.user_count().await?;
    let users = ledger.find_users(&query.query, query.offset, LIMIT).await?;
    let message = render_message(count, &query.query, &users, query.offset);
    let msg = bot
        .send_message(msg.chat.id, message.0)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .reply_markup(message.1)
        .await?;
    Ok(msg.id)
}

pub async fn update_search(
    bot: &Bot,
    _: &User,
    ledger: &Ledger,
    chat_id: ChatId,
    list_params: &UserListParams,
) -> Result<()> {
    let count = ledger.user_count().await?;
    let users = ledger
        .find_users(&list_params.query.query, list_params.query.offset, LIMIT)
        .await?;
    let message = render_message(
        count,
        &list_params.query.query,
        &users,
        list_params.query.offset,
    );
    bot.edit_message_text(chat_id, list_params.message_id, message.0)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .reply_markup(message.1)
        .await?;
    Ok(())
}

fn render_message(
    total_count: u64,
    query: &str,
    users: &[User],
    offset: u64,
) -> (String, InlineKeyboardMarkup) {
    let msg = format!(
        "
    🟣 Всего пользователей: _{}_
    ➖➖➖➖➖➖➖➖➖➖
    🔵 \\- Инструктор
    🟢 \\- Клиент
    🔴 \\- Администратор 

    Что бы найти пользователя, воспользуйтесь поиском\\. Введите имя, фамилию или телефон пользователя\\.\n
    Запрос: _'{}'_
    ",
        total_count,
        escape(query)
    );

    let mut markup = InlineKeyboardMarkup::default();

    for user in users {
        markup = markup.append_row(vec![make_button(user)]);
    }

    let mut raw = vec![];

    if offset > 0 {
        raw.push(InlineKeyboardButton::callback(
            "⬅️",
            SearchCallback::Prev.to_data(),
        ));
    }

    if users.len() == LIMIT as usize {
        raw.push(InlineKeyboardButton::callback(
            "➡️",
            SearchCallback::Next.to_data(),
        ));
    }

    if raw.len() > 0 {
        markup = markup.append_row(raw);
    }
    (msg, markup)
}

fn make_button(user: &User) -> InlineKeyboardButton {
    InlineKeyboardButton::callback(
        format!(
            "{}{} {}",
            user_type(user),
            user.name.first_name,
            user.name.last_name.as_ref().unwrap_or(&"".to_string())
        ),
        SearchCallback::Select(user.user_id.clone()).to_data(),
    )
}
