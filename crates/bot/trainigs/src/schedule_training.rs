use super::{
    create_training::CreateTraining, schedule_process::ScheduleTrainingPreset,
    view_training_proto::ViewProgram,
};
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Dest, View},
};
use chrono::{DateTime, Local};
use eyre::{Error, Result};
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

pub struct ScheduleTraining {
    day: DateTime<Local>,
}

impl ScheduleTraining {
    pub fn new(day: DateTime<Local>) -> Self {
        Self { day }
    }
}

#[async_trait]
impl View for ScheduleTraining {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::EditSchedule)?;
        let (msg, keymap) = render(ctx, &self.day).await?;
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Dest> {
        ctx.ensure(Rule::EditSchedule)?;
        match calldata!(data) {
            Callback::CreateTraining => {
                ctx.ensure(Rule::CreateTraining)?;
                return Ok(CreateTraining::new().into());
            }
            Callback::SelectTraining(id) => {
                ctx.ensure(Rule::EditSchedule)?;
                let id = ObjectId::from_bytes(id);
                let preset = ScheduleTrainingPreset {
                    day: Some(self.day),
                    date_time: None,
                    instructor: None,
                    is_one_time: None,
                };
                return Ok(ViewProgram::new(id, preset).into());
            }
        }
    }
}

async fn render(
    ctx: &mut Context,
    day: &DateTime<Local>,
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let msg = format!(
        "
🤸🏼 Добавить тренировку на день: *{}* _{}_
Выберите тренировку из списка или создайте новую\\.
",
        day.format("%d\\.%m\\.%Y"),
        render_weekday(day)
    );
    let mut keymap = InlineKeyboardMarkup::default();

    let trainings = ctx.ledger.programs.find(&mut ctx.session, None).await?;

    for training in trainings {
        keymap
            .inline_keyboard
            .push(Callback::SelectTraining(training.id.bytes()).btn_row(training.name.clone()));
    }

    keymap
        .inline_keyboard
        .push(Callback::CreateTraining.btn_row("🧘🏼 Создать новую тренировку"));

    Ok((msg, keymap))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    CreateTraining,
    SelectTraining([u8; 12]),
}
