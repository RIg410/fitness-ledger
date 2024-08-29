use eyre::eyre;
use eyre::Result;
use ledger::Ledger;
use storage::user::rights::Rule;
use storage::user::rights::UserRule;
use storage::user::User;
use teloxide::payloads::EditMessageTextSetters as _;
use teloxide::prelude::Requester as _;
use teloxide::types::ChatId;
use teloxide::types::InlineKeyboardButton;
use teloxide::types::InlineKeyboardMarkup;
use teloxide::types::Message;
use teloxide::types::MessageId;
use teloxide::Bot;

use crate::process::profile_menu::format_user_profile;
use crate::state::State;

use super::search::Query;
use super::UserState;

#[derive(Clone, Debug, PartialEq)]
pub enum UserCallback {
    Back,
    BlockUnblock(String),
    Edit(String),
    EditRights(String),
}

impl TryFrom<&str> for UserCallback {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "uc_back" {
            Ok(UserCallback::Back)
        } else if value.starts_with("uc_block:") {
            Ok(UserCallback::BlockUnblock(value[9..].to_string()))
        } else if value.starts_with("uc_edit:") {
            Ok(UserCallback::Edit(value[8..].to_string()))
        } else if value.starts_with("uc_rights:") {
            Ok(UserCallback::EditRights(value[10..].to_string()))
        } else {
            Err(eyre!("Invalid user callback:{}", value))
        }
    }
}

impl UserCallback {
    pub fn to_data(&self) -> String {
        match self {
            UserCallback::Back => "uc_back".to_string(),
            UserCallback::BlockUnblock(id) => format!("uc_block:{}", id),
            UserCallback::Edit(id) => format!("uc_edit:{}", id),
            UserCallback::EditRights(id) => format!("uc_rights:{}", id),
        }
    }
}

pub async fn show_user_profile(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    user_id: String,
    chat_id: ChatId,
    msg_id: MessageId,
) -> Result<()> {
    if !me.rights.has_rule(Rule::User(UserRule::FindUser)) {
        return Err(eyre!("User has no rights to view users"));
    }
    let user = ledger
        .get_user_by_id(&user_id)
        .await?
        .ok_or_else(|| eyre!("User not found"))?;

    let user_info = format_user_profile(&user);
    let mut markup = InlineKeyboardMarkup::default();

    if me.rights.has_rule(Rule::User(UserRule::BlockUser)) && me.user_id != user_id {
        markup = markup.append_row(vec![InlineKeyboardButton::callback(
            if user.is_active {
                "❌ Заблокировать"
            } else {
                "✅ Разблокировать"
            },
            UserCallback::BlockUnblock(user_id.clone()).to_data(),
        )]);
    }

    if me.rights.has_rule(Rule::User(UserRule::EditUserInfo)) {
        markup = markup.append_row(vec![InlineKeyboardButton::callback(
            "✍️ Редактировать",
            UserCallback::Edit(user_id.clone()).to_data(),
        )]);
    }

    if me.rights.has_rule(Rule::User(UserRule::EditUserRights)) {
        markup = markup.append_row(vec![InlineKeyboardButton::callback(
            "🔒 Права",
            UserCallback::EditRights(user_id.clone()).to_data(),
        )]);
    }

    bot.edit_message_text(chat_id, msg_id, user_info)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .reply_markup(markup.append_row(vec![InlineKeyboardButton::callback(
            "⬅️",
            UserCallback::Back.to_data(),
        )]))
        .await?;
    Ok(())
}

pub async fn handle_callback(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    query: (Query, MessageId, String),
    cmd: UserCallback,
    chat_id: ChatId,
) -> Result<Option<State>> {
    match cmd {
        UserCallback::Back => {
            super::search::update_search(bot, me, ledger, &query.0, chat_id, &query.1).await?;
            Ok(Some(State::Users(super::UserState::ShowList((
                query.0, query.1,
            )))))
        }
        UserCallback::BlockUnblock(user_id) => {
            if !me.rights.has_rule(Rule::User(UserRule::BlockUser)) {
                return Err(eyre!("User has no rights to block users"));
            }
            let user = ledger
                .get_user_by_id(&user_id)
                .await?
                .ok_or_else(|| eyre!("User not found"))?;
            ledger.block_user(&user_id, !user.is_active).await?;
            show_user_profile(bot, me, ledger, user_id, chat_id, query.1).await?;
            Ok(Some(State::Users(UserState::SelectUser((
                query.0, query.1, query.2,
            )))))
        }
        UserCallback::Edit(_) => todo!(),
        UserCallback::EditRights(_) => todo!(),
    }
}

pub(crate) async fn handle_message(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    message: &Message,
    state: (Query, MessageId, String),
) -> Result<Option<State>> {
    todo!()
}
