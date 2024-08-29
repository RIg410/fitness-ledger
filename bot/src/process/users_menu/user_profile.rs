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
use teloxide::types::MessageId;
use teloxide::Bot;

use crate::process::profile_menu::format_user_profile;
use crate::state::State;

use super::search::Query;

#[derive(Clone, Debug, PartialEq)]
pub enum UserCallback {
    Back,
}

impl TryFrom<&str> for UserCallback {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "back" {
            Ok(UserCallback::Back)
        } else {
            Err(eyre!("Invalid user callback:{}", value))
        }
    }
}

impl UserCallback {
    pub fn to_data(&self) -> String {
        match self {
            UserCallback::Back => "back".to_string(),
        }
    }
}

pub async fn show_user_profile(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    user_id: String,
    chat_id: ChatId,
    msg_id: MessageId,
) -> Result<()> {
    if !user.rights.has_rule(Rule::User(UserRule::FindUser)) {
        return Err(eyre!("User has no rights to view users"));
    }
    let user = ledger
        .get_user_by_id(&user_id)
        .await?
        .ok_or_else(|| eyre!("User not found"))?;

    let user_info = format_user_profile(&user);
    bot.edit_message_text(chat_id, msg_id, user_info)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .reply_markup(InlineKeyboardMarkup::default().append_row(vec![
            InlineKeyboardButton::callback("⬅️", UserCallback::Back.to_data()),
        ]))
        .await?;
    Ok(())
}

pub async fn handle_callback(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    query: (Query, MessageId, String),
    cmd: UserCallback,
    chat_id: ChatId,
) -> Result<Option<State>> {
    match cmd {
        UserCallback::Back => {
            super::search::update_search(bot, user, ledger, &query.0, chat_id, &query.1).await?;
            Ok(Some(State::Users(super::UserState::ShowList((
                query.0, query.1,
            )))))
        }
    }
}
