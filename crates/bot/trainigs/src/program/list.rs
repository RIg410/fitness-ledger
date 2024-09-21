use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use chrono::Local;
use eyre::{Error, Result};
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

use crate::schedule::ScheduleTrainingPreset;

use super::{create::CreateProgram, view::ProgramView};

pub struct ProgramList {
    preset: Option<ScheduleTrainingPreset>,
}

impl Default for ProgramList {
    fn default() -> Self {
        Self { preset: None }
    }
}

impl ProgramList {
    pub fn new(preset: ScheduleTrainingPreset) -> Self {
        Self {
            preset: Some(preset),
        }
    }
}

#[async_trait]
impl View for ProgramList {
    fn name(&self) -> &'static str {
        "ProgramList"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (msg, keymap) = render(ctx).await?;
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::CreateTraining => {
                ctx.ensure(Rule::CreateTraining)?;
                Ok(CreateProgram::new().into())
            }
            Callback::SelectTraining(id) => {
                let id = ObjectId::from_bytes(id);
                Ok(ProgramView::new(id, self.preset.clone().unwrap_or_default()).into())
            }
        }
    }
}

async fn render(ctx: &mut Context) -> Result<(String, InlineKeyboardMarkup), Error> {
    let msg = format!("Тренировочные программы: 🤸🏼");
    let mut keymap = InlineKeyboardMarkup::default();

    let trainings = ctx.ledger.programs.find(&mut ctx.session, None).await?;

    for training in trainings {
        keymap
            .inline_keyboard
            .push(Callback::SelectTraining(training.id.bytes()).btn_row(training.name));
    }

    if ctx.has_right(Rule::CreateTraining) {
        keymap
            .inline_keyboard
            .push(Callback::CreateTraining.btn_row("🧘🏼 Создать новую тренировку"));
    }
    Ok((msg, keymap))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    CreateTraining,
    SelectTraining([u8; 12]),
}
