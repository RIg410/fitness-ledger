use async_trait::async_trait;
use eyre::{bail, Ok, Result};
use model::rights::Rule;
use strum::EnumIter;
use teloxide::types::{BotCommand, InlineKeyboardButton, InlineKeyboardMarkup, Message};

use crate::{context::Context, state::Widget};

use super::{
    finance::FinanceView,
    subscription::SubscriptionView,
    training::TrainingMainView,
    users::{profile::UserProfile, Query, UsersView},
    View,
};

pub struct MainMenuView;

impl MainMenuView {
    pub async fn send_self(&self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let mut keymap = InlineKeyboardMarkup::default();

        keymap = keymap.append_row(vec![MainMenuItem::Profile.into()]);
        keymap = keymap.append_row(vec![MainMenuItem::Trainings.into()]);
        keymap = keymap.append_row(vec![MainMenuItem::Subscription.into()]);
        if ctx.has_right(Rule::ViewUsers) {
            keymap = keymap.append_row(vec![MainMenuItem::Users.into()]);
        }

        if ctx.has_right(Rule::ViewFinance) {
            keymap = keymap.append_row(vec![MainMenuItem::FinanceView.into()]);
        }

        let id = ctx.send_msg_with_markup("🏠SoulFamily🤸🏼", keymap).await?;
        ctx.update_origin_msg_id(id);
        Ok(())
    }
}

#[async_trait]
impl View for MainMenuView {
    async fn show(&mut self, _: &mut Context) -> Result<(), eyre::Error> {
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

        self.send_self(ctx).await?;
        Ok(Some(match command {
            MainMenuItem::Profile => Box::new(UserProfile::new(ctx.me.tg_id, None)),
            MainMenuItem::Trainings => Box::new(TrainingMainView::default()),
            MainMenuItem::Users => Box::new(UsersView::new(Query::default())),
            MainMenuItem::Subscription => Box::new(SubscriptionView::default()),
            MainMenuItem::FinanceView => Box::new(FinanceView),
            MainMenuItem::Home => return Ok(None),
        }))
    }

    async fn handle_callback(
        &mut self,
        ctx: &mut Context,
        msg: &str,
    ) -> Result<Option<Widget>, eyre::Error> {
        let command = if let Some(command) = MainMenuItem::try_from(msg).ok() {
            command
        } else {
            return Ok(None);
        };
        self.send_self(ctx).await?;
        Ok(Some(match command {
            MainMenuItem::Profile => Box::new(UserProfile::new(ctx.me.tg_id, None)),
            MainMenuItem::Trainings => Box::new(TrainingMainView::default()),
            MainMenuItem::Users => Box::new(UsersView::new(Default::default())),
            MainMenuItem::Subscription => Box::new(SubscriptionView::default()),
            MainMenuItem::Home => Box::new(MainMenuView),
            MainMenuItem::FinanceView => Box::new(FinanceView),
        }))
    }
}

#[derive(EnumIter, Clone, Copy, Debug, PartialEq)]
pub enum MainMenuItem {
    Home,
    Profile,
    Trainings,
    Users,
    Subscription,
    FinanceView,
}

const HOME_DESCRIPTION: &str = "🏠";
const HOME_NAME: &str = "/start";

const PROFILE_DESCRIPTION: &str = "Профиль 🧑";
const PROFILE_NAME: &str = "/profile";

const TRAININGS_DESCRIPTION: &str = "Тренировки 🤸🏻‍♂️";
const TRAININGS_NAME: &str = "/trainings";

const USERS_DESCRIPTION: &str = "Пользователи 👥";
const USERS_NAME: &str = "/users";

const SUBSCRIPTION_DESCRIPTION: &str = "Абонементы 💳";
const SUBSCRIPTION_NAME: &str = "/subscription";

const FINANCE_DESCRIPTION: &str = "Финансы 💰";
const FINANCE_NAME: &str = "/finance";

impl MainMenuItem {
    pub fn description(&self) -> &'static str {
        match self {
            MainMenuItem::Profile => PROFILE_DESCRIPTION,
            MainMenuItem::Trainings => TRAININGS_DESCRIPTION,
            MainMenuItem::Users => USERS_DESCRIPTION,
            MainMenuItem::Subscription => SUBSCRIPTION_DESCRIPTION,
            MainMenuItem::Home => HOME_DESCRIPTION,
            MainMenuItem::FinanceView => FINANCE_DESCRIPTION,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            MainMenuItem::Profile => PROFILE_NAME,
            MainMenuItem::Trainings => TRAININGS_NAME,
            MainMenuItem::Users => USERS_NAME,
            MainMenuItem::Subscription => SUBSCRIPTION_NAME,
            MainMenuItem::Home => HOME_NAME,
            MainMenuItem::FinanceView => FINANCE_NAME,
        }
    }
}

impl From<MainMenuItem> for InlineKeyboardButton {
    fn from(value: MainMenuItem) -> Self {
        InlineKeyboardButton::callback(value.description(), value.name())
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
            TRAININGS_NAME | TRAININGS_DESCRIPTION => Ok(MainMenuItem::Trainings),
            USERS_NAME | USERS_DESCRIPTION => Ok(MainMenuItem::Users),
            SUBSCRIPTION_NAME | SUBSCRIPTION_DESCRIPTION => Ok(MainMenuItem::Subscription),
            HOME_NAME | HOME_DESCRIPTION | "/home" => Ok(MainMenuItem::Home),
            FINANCE_NAME | FINANCE_DESCRIPTION => Ok(MainMenuItem::FinanceView),
            _ => bail!("Unknown command"),
        }
    }
}
