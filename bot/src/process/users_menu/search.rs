use eyre::{eyre, Result};
use ledger::Ledger;
use storage::user::{
    rights::{Rule, TrainingRule, UserRule},
    User,
};
use teloxide::{
    payloads::{EditMessageTextSetters as _, SendMessageSetters as _},
    prelude::Requester as _,
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageId},
    Bot,
};

use crate::{process::users_menu::UserState, state::State};

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
        if value == "next" {
            Ok(SearchCallback::Next)
        } else if value == "prev" {
            Ok(SearchCallback::Prev)
        } else if value.starts_with("select:") {
            Ok(SearchCallback::Select(value[7..].to_string()))
        } else {
            Err(eyre!("Invalid search callback:{}", value))
        }
    }
}

impl SearchCallback {
    pub fn to_data(&self) -> String {
        match self {
            SearchCallback::Next => "next".to_string(),
            SearchCallback::Prev => "prev".to_string(),
            SearchCallback::Select(user_id) => format!("select:{}", user_id),
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

pub async fn handle_callback(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    query: (Query, MessageId),
    cmd: SearchCallback,
) -> Result<Option<State>> {
    if !user.rights.has_rule(Rule::User(UserRule::FindUser)) {
        return Err(eyre!("User has no rights to find users"));
    }

    match cmd {
        SearchCallback::Next => {
            let new_query = Query {
                query: query.0.query.clone(),
                offset: query.0.offset + LIMIT,
            };
            update_search(bot, user, ledger, &new_query, &query.1).await?;
            return Ok(Some(State::Users(UserState::ShowList((
                new_query, query.1,
            )))));
        }
        SearchCallback::Prev => {
            let new_query = Query {
                query: query.0.query.clone(),
                offset: query.0.offset.saturating_sub(LIMIT),
            };
            update_search(bot, user, ledger, &new_query, &query.1).await?;
            return Ok(Some(State::Users(UserState::ShowList((
                new_query, query.1,
            )))));
        }
        SearchCallback::Select(user_id) => {}
    }
    Ok(Some(State::Users(UserState::ShowList((query.0, query.1)))))
}

pub async fn search_users(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    query: &Query,
    msg: &Message,
) -> Result<MessageId> {
    if !user.rights.has_rule(Rule::User(UserRule::FindUser)) {
        return Err(eyre!("User has no rights to find users"));
    }

    let count = ledger.user_count().await?;
    let users = ledger.find_users(&query.query, query.offset, LIMIT).await?;
    let message = render_message(count, &users, query.offset);
    let msg = bot
        .send_message(msg.chat.id, message.0)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .reply_markup(message.1)
        .await?;
    Ok(msg.id)
}

pub async fn update_search(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    query: &Query,
    msg_id: &MessageId,
) -> Result<()> {
    let count = ledger.user_count().await?;
    let users = ledger.find_users(&query.query, query.offset, LIMIT).await?;
    let message = render_message(count, &users, query.offset);
    bot.edit_message_text(ChatId(user.chat_id), msg_id.clone(), message.0)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .reply_markup(message.1)
        .await?;
    Ok(())
}

fn render_message(total_count: u64, users: &[User], offset: u64) -> (String, InlineKeyboardMarkup) {
    let msg = format!(
        "
    🟣 Всего пользователей: _{}_
    ➖➖➖➖➖➖➖➖➖➖
    🔵 \\- Инструктор
    🟢 \\- Клиент
    🔴 \\- Администратор 

    Что бы найти пользователя, воспользуйтесь поиском\\. Введите имя, фамилию или телефон пользователя\\.\n
    ",
        total_count
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
    let user_type = if user.rights.has_rule(Rule::Full) {
        "🔴"
    } else if user.rights.has_rule(Rule::Training(TrainingRule::Train)) {
        "🔵"
    } else {
        "🟢"
    };

    InlineKeyboardButton::callback(
        format!(
            "{}{} {}",
            user_type,
            user.name.first_name,
            user.name.last_name.as_ref().unwrap_or(&"".to_string())
        ),
        SearchCallback::Select(user.user_id.clone()).to_data(),
    )
}
