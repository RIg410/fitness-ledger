pub mod rights;

use eyre::eyre;
use eyre::Result;
use ledger::Ledger;
use log::warn;
use rights::show_user_rights;
use storage::user::rights::Rule;
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

use super::SelectedUser;
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
    me.rights.ensure(Rule::ViewUsers)?;
    let user = ledger
        .get_user_by_tg_id(&user_id)
        .await?
        .ok_or_else(|| eyre!("User not found"))?;

    let user_info = format_user_profile(&user);
    let mut markup = InlineKeyboardMarkup::default();

    if me.rights.has_rule(Rule::BlockUser) && me.user_id != user_id {
        markup = markup.append_row(vec![InlineKeyboardButton::callback(
            if user.is_active {
                "❌ Заблокировать"
            } else {
                "✅ Разблокировать"
            },
            UserCallback::BlockUnblock(user_id.clone()).to_data(),
        )]);
    }

    if me.rights.has_rule(Rule::EditUserInfo) {
        markup = markup.append_row(vec![InlineKeyboardButton::callback(
            "✍️ Редактировать",
            UserCallback::Edit(user_id.clone()).to_data(),
        )]);
    }

    if me.rights.has_rule(Rule::EditUserRights) {
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
    query: SelectedUser,
    cmd: UserCallback,
    chat_id: ChatId,
) -> Result<Option<State>> {
    match cmd {
        UserCallback::Back => {
            super::search::update_search(bot, me, ledger, chat_id, &query.list).await?;
            UserState::ShowList(query.list).into()
        }
        UserCallback::BlockUnblock(user_id) => {
            me.rights.ensure(Rule::BlockUser)?;
            let user = ledger
                .get_user_by_tg_id(&user_id)
                .await?
                .ok_or_else(|| eyre!("User not found"))?;
            ledger.block_user(&user_id, !user.is_active).await?;
            show_user_profile(bot, me, ledger, user_id, chat_id, query.list.message_id).await?;
            UserState::SelectUser(query).into()
        }
        UserCallback::Edit(_) => {
            warn!("Edit user info not implemented");
            UserState::SelectUser(query).into()
        }
        UserCallback::EditRights(user_id) => {
            me.rights.ensure(Rule::EditUserRights)?;
            show_user_rights(bot, me, ledger, user_id, chat_id, query).await
        }
    }
}

pub(crate) async fn handle_message(
    bot: &Bot,
    _: &User,
    _: &Ledger,
    msg: &Message,
    state: SelectedUser,
) -> Result<Option<State>> {
    bot.delete_message(msg.chat.id, msg.id).await?;
    UserState::SelectUser(state).into()
}
