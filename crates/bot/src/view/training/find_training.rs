use super::{schedule_process::ScheduleTrainingPreset, view_training_proto::ViewProgram, View};
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

#[derive(Default)]
pub struct FindTraining {
    go_back: Option<Widget>,
}

impl FindTraining {
    pub fn new(go_back: Option<Widget>) -> Self {
        Self { go_back }
    }
}

#[async_trait]
impl View for FindTraining {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (msg, keyboard) = render(ctx).await?;
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
        let cb = if let Some(cb) = Callback::from_data(data) {
            cb
        } else {
            return Ok(None);
        };
        match cb {
            Callback::SelectTraining(id) => {
                let id = ObjectId::from_bytes(id);
                let back = FindTraining::new(self.go_back.take());
                let view = Box::new(ViewProgram::new(
                    id,
                    ScheduleTrainingPreset::default(),
                    Some(Box::new(back)),
                ));
                Ok(Some(view))
            }
            Callback::Back => Ok(self.go_back.take()),
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
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            proto.name.clone(),
            Callback::SelectTraining(proto.id.bytes()).to_data(),
        )]);
    }
    Ok((msg, keymap))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    SelectTraining([u8; 12]),
    Back,
}
