use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::{Error, Result};
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

#[derive(Default)]
pub struct ProgramList;

#[async_trait]
impl View for ProgramList {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (msg, keymap) = render(ctx).await?;
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::CreateTraining => {
                // ctx.ensure(Rule::CreateTraining)?;
                // return Ok(Some(CreateTraining::new().boxed()));
                todo!()
            }
            Callback::SelectTraining(id) => {
                // let id = ObjectId::from_bytes(id);
                // let preset = ScheduleTrainingPreset {
                //     day: None,
                //     date_time: None,
                //     instructor: None,
                //     is_one_time: None,
                // };
                // return Ok(Some(ViewProgram::new(id, preset).boxed()));
                todo!()
            }
        }
        Ok(Jmp::None)
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
