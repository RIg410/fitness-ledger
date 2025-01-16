use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::Result;
use model::user::User;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use super::{render_msg, PersonalTrainingPreset};

#[derive(Default)]
pub struct SetInstructor {
    preset: PersonalTrainingPreset,
}

impl SetInstructor {
    pub fn new(preset: PersonalTrainingPreset) -> Self {
        Self { preset }
    }
}

#[async_trait]
impl View for SetInstructor {
    fn name(&self) -> &'static str {
        "SetInstructor"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (msg, keymap) = render(ctx, &self.preset).await?;
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::SelectInstructor(instructor_id) => {
                let instructor = ctx
                    .ledger
                    .users
                    .get(&mut ctx.session, ObjectId::from_bytes(instructor_id))
                    .await?
                    .ok_or_else(|| eyre::eyre!("Instructor not found"))?;
                self.preset.instructor = Some(instructor.id);
                return Ok(self.preset.into_next_view().into());
            }
        }
    }
}

async fn render(
    ctx: &mut Context,
    preset: &PersonalTrainingPreset,
) -> Result<(String, InlineKeyboardMarkup)> {
    let mut keymap = InlineKeyboardMarkup::default();

    let instructors = ctx.ledger.users.instructors(&mut ctx.session).await?;
    for instructor in instructors {
        keymap
            .inline_keyboard
            .push(vec![make_instructor_button(&instructor)]);
    }
    let message = render_msg(ctx, preset, "🫰Выберите инструктора").await?;
    Ok((message, keymap))
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
    Callback::SelectInstructor(instructor.id.bytes()).button(name)
}

#[derive(Debug, Serialize, Deserialize)]
enum Callback {
    SelectInstructor([u8; 12]),
}
