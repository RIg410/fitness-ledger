use super::{ScheduleTrainingPreset, View};
use crate::{callback_data::Calldata, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use model::{program::Program, user::User};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::Requester as _,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

#[derive(Default)]
pub struct SetInstructor {
    id: ObjectId,
    preset: Option<ScheduleTrainingPreset>,
    go_back: Option<Widget>,
}

impl SetInstructor {
    pub fn new(id: ObjectId, preset: ScheduleTrainingPreset, go_back: Widget) -> Self {
        Self {
            id,
            preset: Some(preset),
            go_back: Some(go_back),
        }
    }
}

#[async_trait]
impl View for SetInstructor {
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

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        ctx.bot.delete_message(message.chat.id, message.id).await?;
        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        match InstructorCallback::from_data(data)? {
            InstructorCallback::SelectInstructor(instructor_id) => {
                let instructor = ctx
                    .ledger
                    .users
                    .get_by_tg_id(&mut ctx.session, instructor_id)
                    .await?
                    .ok_or_else(|| eyre::eyre!("Instructor not found"))?;
                let mut preset = self.preset.take().unwrap();
                preset.instructor = Some(instructor.tg_id);
                return Ok(Some(
                    preset.into_next_view(self.id, self.go_back.take().unwrap()),
                ));
            }
        }
    }
}

async fn render(ctx: &mut Context, training: &Program) -> Result<(String, InlineKeyboardMarkup)> {
    let msg = format!(
        "🫰Выберите инструктора для тренировки *{}*",
        escape(&training.name)
    );
    let mut markup = InlineKeyboardMarkup::default();

    let instructors = ctx.ledger.users.instructors(&mut ctx.session).await?;
    for instructor in instructors {
        markup
            .inline_keyboard
            .push(vec![make_instructor_button(&instructor)]);
    }

    Ok((msg, markup))
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
    InlineKeyboardButton::callback(
        name,
        InstructorCallback::SelectInstructor(instructor.tg_id).to_data(),
    )
}

#[derive(Debug, Serialize, Deserialize)]
enum InstructorCallback {
    SelectInstructor(i64),
}
