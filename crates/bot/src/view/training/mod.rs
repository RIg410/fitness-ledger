pub mod create_training;
pub mod find_training;
pub mod me_training;
pub mod schedule_process;
pub mod schedule_training;
pub mod view_training_proto;

use super::{calendar::CalendarView, menu::MainMenuItem, View};
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use find_training::FindTraining;
use me_training::MyTrainings;
use model::ids::WeekId;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

#[derive(Default)]
pub struct TrainingMainView;

#[async_trait]
impl View for TrainingMainView {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (msg, keyboard) = render();
        ctx.edit_origin(&msg, keyboard).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        ctx.delete_msg(message.id).await?;
        Ok(None)
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Option<Widget>> {
        match ScheduleLendingCallback::from_data(data)? {
            ScheduleLendingCallback::MyTrainings => {
                return Ok(Some(Box::new(MyTrainings::default())))
            }
            ScheduleLendingCallback::Schedule => {
                let widget = Box::new(CalendarView::new(
                    WeekId::default(),
                    Some(Box::new(TrainingMainView)),
                    None,
                    None,
                ));
                return Ok(Some(widget));
            }
            ScheduleLendingCallback::FindTraining => {
                let widget = Box::new(FindTraining::default());
                return Ok(Some(widget));
            }
        }
    }
}

pub fn render() -> (String, InlineKeyboardMarkup) {
    let msg = "🤸🏻‍♂️  Подберем тренировку для вас:".to_owned();
    let mut keymap = InlineKeyboardMarkup::default();
    keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
        "🫶🏻 Мои тренировки",
        ScheduleLendingCallback::MyTrainings.to_data(),
    )]);
    keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
        "📅  Календарь",
        ScheduleLendingCallback::Schedule.to_data(),
    )]);
    keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
        "🔍 Найти тренировку",
        ScheduleLendingCallback::FindTraining.to_data(),
    )]);

    keymap = keymap.append_row(vec![MainMenuItem::Home.into()]);
    (msg, keymap)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScheduleLendingCallback {
    MyTrainings,
    Schedule,
    FindTraining,
}
