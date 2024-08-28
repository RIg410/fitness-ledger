use crate::state::{self, State};
use eyre::Result;
use ledger::Ledger;
use storage::user::User;
use teloxide::{payloads::SendMessageSetters as _, prelude::Requester as _, types::Message, Bot};

#[derive(Clone, Debug, Default)]
pub enum ProfileState {
    #[default]
    Start,
}

pub async fn go_to_profile(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    msg: &Message,
) -> Result<Option<State>> {
    println!("go_to_profile");
    let text = format_user_profile(user);
    bot.send_message(msg.chat.id, text)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    Ok(Some(State::Profile(ProfileState::Start)))
}

pub async fn handle_message(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    msg: &Message,
    state: ProfileState,
) -> Result<Option<State>> {
    Ok(None)
}

fn format_user_profile(user: &User) -> String {
    format!(
        "
    🟣 Пользователь : {}
    *bold text*\n\n_italic text_
    ",
        user.name.tg_user_name.as_ref().unwrap_or(&"Unknown".to_string())
    )
}
