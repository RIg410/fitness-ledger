use async_trait::async_trait;
use eyre::{bail, Ok, Result};
use storage::user::rights::Rule;
use strum::{EnumIter, IntoEnumIterator};
use teloxide::types::{BotCommand, KeyboardButton, KeyboardMarkup, Message};

use crate::{context::Context, state::Widget};

use super::{
    profile::UserProfile, schedule::ScheduleView, subscription::SubscriptionView, users::UsersView,
    View,
};

const COLUMNS: usize = 2;

pub struct MainMenuView;

#[async_trait]
impl View for MainMenuView {
    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let mut keymap = Vec::<Vec<KeyboardButton>>::with_capacity(3);

        for item in MainMenuItem::iter() {
            if MainMenuItem::Users == item && !ctx.has_right(Rule::ViewUsers) {
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
        let id = ctx.send_replay_markup("\\.", keymap).await?;
        ctx.update_origin_msg_id(id);
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Option<Widget>, eyre::Error> {
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

        let id = ctx.send_msg("\\.").await?;
        ctx.update_origin_msg_id(id);
        Ok(Some(match command {
            MainMenuItem::Profile => Box::new(UserProfile::default()),
            MainMenuItem::Schedule => Box::new(ScheduleView::default()),
            MainMenuItem::Users => Box::new(UsersView::default()),
            MainMenuItem::Subscription => Box::new(SubscriptionView::default()),
        }))
    }

    async fn handle_callback(
        &mut self,
        _: &mut Context,
        _: &str,
    ) -> Result<Option<Widget>, eyre::Error> {
        Ok(None)
    }
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
