use super::{schedule_process::ScheduleTrainingPreset, view_training_proto::ViewProgram};
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::Result;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

#[derive(Default)]
pub struct FindTraining;

#[async_trait]
impl View for FindTraining {
    fn name(&self) -> &'static str {
        "FindTraining"
    }
    
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (msg, keyboard) = render(ctx).await?;
        ctx.edit_origin(&msg, keyboard).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::SelectTraining(id) => {
                let id = ObjectId::from_bytes(id);
                Ok(ViewProgram::new(id, ScheduleTrainingPreset::default()).into())
            }
        }
    }
}

async fn render(ctx: &mut Context) -> Result<(String, InlineKeyboardMarkup)> {
    let mut msg = "🤸🏻‍♂️  Подберем тренировку для вас:".to_owned();
    let trainings = ctx.ledger.programs.get_all(&mut ctx.session).await?;
    if trainings.is_empty() {
        msg.push_str("\n\nУ нас пока нет тренировок");
    } else {
        msg.push_str("\n\nвот что у нас есть:");
    }
    let mut keymap = InlineKeyboardMarkup::default();
    for proto in trainings {
        keymap = keymap
            .append_row(Callback::SelectTraining(proto.id.bytes()).btn_row(proto.name.clone()));
    }
    Ok((msg, keymap))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    SelectTraining([u8; 12]),
}
