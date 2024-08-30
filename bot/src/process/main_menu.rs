use eyre::{bail, Ok, Result};
use ledger::Ledger;
use storage::user::{rights::Rule, User};
use strum::{EnumIter, IntoEnumIterator};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{BotCommand, ChatId, KeyboardButton, KeyboardMarkup, Message},
    Bot,
};

use crate::state::State;

use super::{
    profile_menu::go_to_profile, schedule_menu::go_to_schedule_lending, users_menu::go_to_users,
};

const COLUMNS: usize = 2;

pub async fn handle_message(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    msg: &Message,
) -> Result<Option<State>> {
    let text = if let Some(text) = msg.text() {
        text
    } else {
        return Ok(None);
    };

    let command = if let Some(command) = MainMenuItem::try_from(text).ok() {
        command
    } else {
        return Ok(None);
    };

    match command {
        MainMenuItem::Profile => go_to_profile(bot, user, ledger, msg).await,
        MainMenuItem::Schedule => go_to_schedule_lending(bot, user, ledger, msg).await,
        MainMenuItem::Users => go_to_users(bot, user, ledger, msg).await,
        MainMenuItem::Subscription => bail!("Not implemented"),
    }
}

pub async fn show_commands(bot: &Bot, user: &User) -> Result<()> {
    let mut keymap = Vec::<Vec<KeyboardButton>>::with_capacity(3);

    for item in MainMenuItem::iter() {
        if MainMenuItem::Users == item && !user.rights.has_rule(Rule::ViewUsers) {
            continue;
        }

        if let Some(last) = keymap.last() {
            if last.len() == COLUMNS {
                keymap.push(Vec::with_capacity(COLUMNS));
            }
        } else {
            keymap.push(Vec::with_capacity(COLUMNS));
        }
        keymap
            .last_mut()
            .unwrap()
            .push(KeyboardButton::new(item.description()));
    }
    let keymap = KeyboardMarkup::new(keymap);

    bot.send_message(ChatId(user.chat_id), "🏠")
        .reply_markup(keymap)
        .await?;
    Ok(())
}

#[derive(EnumIter, Clone, Copy, Debug, PartialEq)]
pub enum MainMenuItem {
    Profile,
    Schedule,
    Users,
    Subscription,
}

const PROFILE_DESCRIPTION: &str = "Профиль 🧑";
const PROFILE_NAME: &str = "/profile";

const SCHEDULE_DESCRIPTION: &str = "Расписание 📅";
const SCHEDULE_NAME: &str = "/schedule";

const USERS_DESCRIPTION: &str = "Пользователи 👥";
const USERS_NAME: &str = "/users";

const SUBSCRIPTION_DESCRIPTION: &str = "Абонементы 💳";
const SUBSCRIPTION_NAME: &str = "/subscription";

impl MainMenuItem {
    pub fn description(&self) -> &'static str {
        match self {
            MainMenuItem::Profile => PROFILE_DESCRIPTION,
            MainMenuItem::Schedule => SCHEDULE_DESCRIPTION,
            MainMenuItem::Users => USERS_DESCRIPTION,
            MainMenuItem::Subscription => SUBSCRIPTION_DESCRIPTION,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            MainMenuItem::Profile => PROFILE_NAME,
            MainMenuItem::Schedule => SCHEDULE_NAME,
            MainMenuItem::Users => USERS_NAME,
            MainMenuItem::Subscription => SUBSCRIPTION_NAME,
        }
    }
}

impl From<MainMenuItem> for BotCommand {
    fn from(value: MainMenuItem) -> Self {
        BotCommand {
            command: value.name().to_string(),
            description: value.description().to_string(),
        }
    }
}

impl TryFrom<&str> for MainMenuItem {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            PROFILE_NAME | PROFILE_DESCRIPTION => Ok(MainMenuItem::Profile),
            SCHEDULE_NAME | SCHEDULE_DESCRIPTION => Ok(MainMenuItem::Schedule),
            USERS_NAME | USERS_DESCRIPTION => Ok(MainMenuItem::Users),
            SUBSCRIPTION_NAME | SUBSCRIPTION_DESCRIPTION => Ok(MainMenuItem::Subscription),
            _ => bail!("Unknown command"),
        }
    }
}
