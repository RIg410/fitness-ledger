use super::ScheduleTrainingPreset;
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::Result;
use model::{program::Program, user::User};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::markdown::escape,
};

#[derive(Default)]
pub struct SetInstructor {
    id: ObjectId,
    preset: Option<ScheduleTrainingPreset>,
}

impl SetInstructor {
    pub fn new(id: ObjectId, preset: ScheduleTrainingPreset) -> Self {
        Self {
            id,
            preset: Some(preset),
        }
    }
}

#[async_trait]
impl View for SetInstructor {
    fn name(&self) -> &'static str {
        "SetInstructor"
    }
    
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let training = ctx
            .ledger
            .programs
            .get_by_id(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let (msg, keymap) = render(ctx, &training).await?;
        ctx.send_msg_with_markup(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::SelectInstructor(instructor_id) => {
                let instructor = ctx
                    .ledger
                    .users
                    .get_by_tg_id(&mut ctx.session, instructor_id)
                    .await?
                    .ok_or_else(|| eyre::eyre!("Instructor not found"))?;
                let mut preset = self.preset.take().unwrap();
                preset.instructor = Some(instructor.tg_id);
                return Ok(preset.into_next_view(self.id).into());
            }
        }
    }
}

async fn render(ctx: &mut Context, training: &Program) -> Result<(String, InlineKeyboardMarkup)> {
    let msg = format!(
        "🫰Выберите инструктора для тренировки *{}*",
        escape(&training.name)
    );
    let mut keymap = InlineKeyboardMarkup::default();

    let instructors = ctx.ledger.users.instructors(&mut ctx.session).await?;
    for instructor in instructors {
        keymap
            .inline_keyboard
            .push(vec![make_instructor_button(&instructor)]);
    }
    Ok((msg, keymap))
}

fn make_instructor_button(instructor: &User) -> InlineKeyboardButton {
    let name = format!(
        "{} {}",
        instructor.name.first_name,
        instructor
            .name
            .last_name
            .as_ref()
            .unwrap_or(&"".to_string())
    );
    Callback::SelectInstructor(instructor.tg_id).button(name)
}

#[derive(Debug, Serialize, Deserialize)]
enum Callback {
    SelectInstructor(i64),
}
