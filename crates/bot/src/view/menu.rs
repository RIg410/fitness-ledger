use super::{
    couching::CouchingView,
    finance::FinanceView,
    logs::LogsView,
    subscription::SubscriptionView,
    training::TrainingMainView,
    users::{profile::UserProfile, Query, UsersView},
    View,
};
use crate::{context::Context, state::Widget};
use async_trait::async_trait;
use eyre::{bail, Ok, Result};
use model::rights::Rule;
use strum::EnumIter;
use teloxide::types::{BotCommand, InlineKeyboardButton, InlineKeyboardMarkup, Message};

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

        if ctx.has_right(Rule::ViewLogs) {
            keymap = keymap.append_row(vec![MainMenuItem::LogView.into()]);
        }

        if ctx.has_right(Rule::CouchingView) {
            keymap = keymap.append_row(vec![MainMenuItem::Coaching.into()]);
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
            MainMenuItem::Profile => UserProfile::new(ctx.me.tg_id, None).boxed(),
            MainMenuItem::Trainings => TrainingMainView::default().boxed(),
            MainMenuItem::Users => UsersView::new(Query::default()).boxed(),
            MainMenuItem::Subscription => SubscriptionView::default().boxed(),
            MainMenuItem::FinanceView => FinanceView.boxed(),
            MainMenuItem::LogView => LogsView::default().boxed(),
            MainMenuItem::Coaching => CouchingView::default().boxed(),
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
            MainMenuItem::Profile => UserProfile::new(ctx.me.tg_id, None).boxed(),
            MainMenuItem::Trainings => TrainingMainView::default().boxed(),
            MainMenuItem::Users => UsersView::new(Default::default()).boxed(),
            MainMenuItem::Subscription => SubscriptionView::default().boxed(),
            MainMenuItem::Home => MainMenuView.boxed(),
            MainMenuItem::FinanceView => FinanceView.boxed(),
            MainMenuItem::LogView => LogsView::default().boxed(),
            MainMenuItem::Coaching => CouchingView::default().boxed(),
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
    LogView,
    Coaching,
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

const LOG_DESCRIPTION: &str = "Логи 📜";
const LOG_NAME: &str = "/log";

const COACHING_DESCRIPTION: &str = "Тренерская 🧘🏼‍♂️";
const COACHING_NAME: &str = "/coaching";

impl MainMenuItem {
    pub fn description(&self) -> &'static str {
        match self {
            MainMenuItem::Profile => PROFILE_DESCRIPTION,
            MainMenuItem::Trainings => TRAININGS_DESCRIPTION,
            MainMenuItem::Users => USERS_DESCRIPTION,
            MainMenuItem::Subscription => SUBSCRIPTION_DESCRIPTION,
            MainMenuItem::Home => HOME_DESCRIPTION,
            MainMenuItem::FinanceView => FINANCE_DESCRIPTION,
            MainMenuItem::LogView => LOG_DESCRIPTION,
            MainMenuItem::Coaching => COACHING_DESCRIPTION,
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
            MainMenuItem::LogView => LOG_NAME,
            MainMenuItem::Coaching => COACHING_NAME,
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
            LOG_NAME | LOG_DESCRIPTION => Ok(MainMenuItem::LogView),
            COACHING_NAME | COACHING_DESCRIPTION => Ok(MainMenuItem::Coaching),
            _ => bail!("Unknown command"),
        }
    }
}
