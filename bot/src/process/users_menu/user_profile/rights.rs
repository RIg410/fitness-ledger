use crate::{
    process::users_menu::{user_type, SelectedUser, UserState},
    state::State,
};
use eyre::eyre;
use eyre::Result;
use ledger::Ledger;
use storage::user::{
    rights::{Rule, UserRule},
    User,
};
use teloxide::{
    payloads::EditMessageTextSetters as _,
    prelude::Requester,
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup},
    Bot,
};

use super::show_user_profile;

pub enum UserRightsCallback {
    Back,
    EditRule(u32, bool),
}

impl UserRightsCallback {
    pub fn to_data(&self) -> String {
        match self {
            UserRightsCallback::Back => "rc_back".to_string(),
            UserRightsCallback::EditRule(id, is_active) => format!("rc_edit:{}:{}", id, is_active),
        }
    }
}

impl UserRightsCallback {
    pub fn try_from(value: &str) -> Result<Self> {
        let parts: Vec<&str> = value.split(':').collect();
        match parts.as_slice() {
            ["rc_back"] => Ok(UserRightsCallback::Back),
            ["rc_edit", id, is_active] => Ok(UserRightsCallback::EditRule(
                id.parse()?,
                is_active.parse()?,
            )),
            _ => Err(eyre!("Invalid rights callback: {}", value)),
        }
    }
}

pub async fn handle_callback(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    query: SelectedUser,
    cmd: UserRightsCallback,
    chat_id: ChatId,
) -> Result<Option<State>> {
    match cmd {
        UserRightsCallback::Back => {
            show_user_profile(
                bot,
                me,
                ledger,
                query.user_id.clone(),
                chat_id,
                query.list.message_id.clone(),
            )
            .await?;
            UserState::SelectUser(query).into()
        }
        UserRightsCallback::EditRule(rule_id, is_active) => {
            if !me.rights.has_rule(Rule::User(UserRule::EditUserRights)) {
                log::warn!("User {} has no rights to edit user rights", me.user_id);
                return Ok(None);
            }

            let rule = Rule::try_from(rule_id)?;
            ledger
                .edit_user_rule(&query.user_id, rule, is_active)
                .await?;

            let user = ledger
                .get_user_by_tg_id(&query.user_id)
                .await?
                .ok_or_else(|| eyre!("User not found"))?;

            render_user_rights(bot, &user, chat_id, &query).await?;
            UserState::UserRights(query).into()
        }
    }
}

pub async fn show_user_rights(
    bot: &Bot,
    _: &User,
    ledger: &Ledger,
    user_id: String,
    chat_id: ChatId,
    query: SelectedUser,
) -> Result<Option<State>> {
    let user = ledger
        .get_user_by_tg_id(&user_id)
        .await?
        .ok_or_else(|| eyre!("User not found"))?;
    render_user_rights(bot, &user, chat_id, &query).await?;
    UserState::UserRights(query).into()
}

async fn render_user_rights(
    bot: &Bot,
    user: &User,
    chat_id: ChatId,
    state: &SelectedUser,
) -> Result<()> {
    let mut msg = format!("{} 🔒Права:", user_type(user));
    let mut keyboard = InlineKeyboardMarkup::default();

    if !user.rights.has_rule(Rule::Full) {
        for (rule, is_active) in user.rights.get_all_rules().iter() {
            if rule == &Rule::Full {
                continue;
            }

            keyboard = keyboard.append_row(vec![InlineKeyboardButton::callback(
                format!(
                    "{} {}",
                    rule.name(),
                    if *is_active {
                        "✅"
                    } else {
                        "❌"
                    }
                ),
                UserRightsCallback::EditRule(rule.id(), !is_active).to_data(),
            )]);
        }
    } else {
        msg.push_str("\n\nПользователь имеет полные права");
    }

    keyboard = keyboard.append_row(vec![InlineKeyboardButton::callback(
        "⬅️",
        UserRightsCallback::Back.to_data(),
    )]);

    bot.edit_message_text(chat_id, state.list.message_id, msg)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}
