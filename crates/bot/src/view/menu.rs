// use super::{
//     calendar::CalendarView,
//     couching::{couch_list::CouchingList, programs_list::ProgramList},
//     finance::FinanceView,
//     logs::LogsView,
//     subscription::SubscriptionView,
//     users::{profile::UserProfile, Query, UsersView},
//     View,
// };
//use crate::{context::Context, state::Widget};
use async_trait::async_trait;
use bot_core::{
    context::Context,
    widget::{Goto, View},
};
use eyre::{bail, Ok, Result};
use model::rights::Rule;
use strum::EnumIter;
use teloxide::types::{BotCommand, InlineKeyboardButton, InlineKeyboardMarkup, Message};

use super::signup::{self, SignUpView};

pub struct MainMenuView;

impl MainMenuView {
    pub async fn send_self(&self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let mut keymap = InlineKeyboardMarkup::default();

        keymap = keymap.append_row(vec![MainMenuItem::Profile.into()]);
        keymap = keymap.append_row(vec![MainMenuItem::Schedule.into()]);
        keymap = keymap.append_row(vec![MainMenuItem::Subscription.into()]);
        keymap = keymap.append_row(vec![MainMenuItem::Coach.into()]);
        keymap = keymap.append_row(vec![MainMenuItem::Programs.into()]);

        if ctx.has_right(Rule::ViewUsers) {
            keymap = keymap.append_row(vec![MainMenuItem::Users.into()]);
        }
        if ctx.has_right(Rule::ViewFinance) {
            keymap = keymap.append_row(vec![MainMenuItem::FinanceView.into()]);
        }

        if ctx.has_right(Rule::ViewLogs) {
            keymap = keymap.append_row(vec![MainMenuItem::LogView.into()]);
        }

        let id = ctx
            .send_msg_with_markup("🏠SoulFamily       🤸🏼", keymap)
            .await?;
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
    ) -> Result<Goto, eyre::Error> {
        if !ctx.is_real_user {
            return Ok(SignUpView::default().into());
        }
        let text = if let Some(text) = msg.text() {
            text
        } else {
            return Ok(Goto::None);
        };

        let command = if let Some(command) = MainMenuItem::try_from(text).ok() {
            command
        } else {
            return Ok(Goto::None);
        };

        self.send_self(ctx).await?;
        // Ok(Some(match command {
        //     MainMenuItem::Profile => UserProfile::new(ctx.me.tg_id).boxed(),
        //     MainMenuItem::Schedule => CalendarView::default().boxed(),
        //     MainMenuItem::Users => UsersView::new(Query::default()).boxed(),
        //     MainMenuItem::Subscription => SubscriptionView::default().boxed(),
        //     MainMenuItem::FinanceView => FinanceView.boxed(),
        //     MainMenuItem::LogView => LogsView::default().boxed(),
        //     MainMenuItem::Coach => CouchingList::new().boxed(),
        //     MainMenuItem::Home => MainMenuView.boxed(),
        //     MainMenuItem::Programs => ProgramList::new().boxed(),
        // }))
        Ok(Goto::None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, msg: &str) -> Result<Goto, eyre::Error> {
        if !ctx.is_real_user {
            return Ok(SignUpView::default().into());
        }

        let command = if let Some(command) = MainMenuItem::try_from(msg).ok() {
            command
        } else {
            return Ok(Goto::None);
        };
        self.send_self(ctx).await?;
        // Ok(Some(match command {
        //     MainMenuItem::Profile => UserProfile::new(ctx.me.tg_id).boxed(),
        //     MainMenuItem::Schedule => CalendarView::default().boxed(),
        //     MainMenuItem::Users => UsersView::new(Default::default()).boxed(),
        //     MainMenuItem::Subscription => SubscriptionView::default().boxed(),
        //     MainMenuItem::Home => MainMenuView.boxed(),
        //     MainMenuItem::FinanceView => FinanceView.boxed(),
        //     MainMenuItem::LogView => LogsView::default().boxed(),
        //     MainMenuItem::Coach => CouchingList::new().boxed(),
        //     MainMenuItem::Programs => ProgramList::new().boxed(),
        // }))
        Ok(Goto::None)
    }

    fn allow_unsigned_user(&self) -> bool {
        true
    }
}

#[derive(EnumIter, Clone, Copy, Debug, PartialEq)]
pub enum MainMenuItem {
    Home,
    Profile,
    Schedule,
    Users,
    Subscription,
    FinanceView,
    LogView,
    Coach,
    Programs,
}

const HOME_DESCRIPTION: &str = "🏠";
const HOME_NAME: &str = "/start";

const PROFILE_DESCRIPTION: &str = "Профиль 🧑";
const PROFILE_NAME: &str = "/profile";

const TRAININGS_DESCRIPTION: &str = "Расписание 📅";
const TRAININGS_NAME: &str = "/schedule";

const SUBSCRIPTION_DESCRIPTION: &str = "Абонементы 💳";
const SUBSCRIPTION_NAME: &str = "/subscription";

const COUCH_DESCRIPTION: &str = "Наши инструкторы ❤️";
const COUCH_NAME: &str = "/couch";

const PROGRAM_DESCRIPTION: &str = "Наши программы 💪🏼";
const PROGRAM_NAME: &str = "/program";

const USERS_DESCRIPTION: &str = "Пользователи 👥";
const USERS_NAME: &str = "/users";

const FINANCE_DESCRIPTION: &str = "Финансы 💰";
const FINANCE_NAME: &str = "/finance";

const LOG_DESCRIPTION: &str = "Логи 📜";
const LOG_NAME: &str = "/log";

impl MainMenuItem {
    pub fn description(&self) -> &'static str {
        match self {
            MainMenuItem::Profile => PROFILE_DESCRIPTION,
            MainMenuItem::Schedule => TRAININGS_DESCRIPTION,
            MainMenuItem::Users => USERS_DESCRIPTION,
            MainMenuItem::Subscription => SUBSCRIPTION_DESCRIPTION,
            MainMenuItem::Home => HOME_DESCRIPTION,
            MainMenuItem::FinanceView => FINANCE_DESCRIPTION,
            MainMenuItem::LogView => LOG_DESCRIPTION,
            MainMenuItem::Coach => COUCH_DESCRIPTION,
            MainMenuItem::Programs => PROGRAM_DESCRIPTION,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            MainMenuItem::Profile => PROFILE_NAME,
            MainMenuItem::Schedule => TRAININGS_NAME,
            MainMenuItem::Users => USERS_NAME,
            MainMenuItem::Subscription => SUBSCRIPTION_NAME,
            MainMenuItem::Home => HOME_NAME,
            MainMenuItem::FinanceView => FINANCE_NAME,
            MainMenuItem::LogView => LOG_NAME,
            MainMenuItem::Coach => COUCH_NAME,
            MainMenuItem::Programs => PROGRAM_NAME,
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
            TRAININGS_NAME | TRAININGS_DESCRIPTION => Ok(MainMenuItem::Schedule),
            USERS_NAME | USERS_DESCRIPTION => Ok(MainMenuItem::Users),
            SUBSCRIPTION_NAME | SUBSCRIPTION_DESCRIPTION => Ok(MainMenuItem::Subscription),
            HOME_NAME | HOME_DESCRIPTION | "/home" => Ok(MainMenuItem::Home),
            FINANCE_NAME | FINANCE_DESCRIPTION => Ok(MainMenuItem::FinanceView),
            LOG_NAME | LOG_DESCRIPTION => Ok(MainMenuItem::LogView),
            COUCH_NAME | COUCH_DESCRIPTION => Ok(MainMenuItem::Coach),
            PROGRAM_NAME | PROGRAM_DESCRIPTION => Ok(MainMenuItem::Programs),
            _ => bail!("Unknown command"),
        }
    }
}
