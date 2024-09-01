use super::{schedule_process::ScheduleTrainingPreset, View};
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use storage::{training::model::TrainingProto, user::rights::Rule};
use teloxide::{
    prelude::Requester as _,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct ViewTrainingProto {
    id: ObjectId,
    go_back: Option<Widget>,
    preset: ScheduleTrainingPreset,
}

impl ViewTrainingProto {
    pub fn new(id: ObjectId, preset: ScheduleTrainingPreset, go_back: Option<Widget>) -> Self {
        Self {
            id,
            go_back,
            preset,
        }
    }
}

#[async_trait]
impl View for ViewTrainingProto {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let training = ctx
            .ledger
            .get_training_by_id(self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let (text, keymap) = render(ctx, &training, self.go_back.is_some()).await?;
        ctx.edit_origin(&text, keymap).await?;
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
        match TrainingProtoCallback::from_data(data)? {
            TrainingProtoCallback::Schedule => {
                ctx.ensure(Rule::EditSchedule)?;
                let preset = self.preset.clone();
                let view = preset.into_next_view(
                    self.id,
                    Box::new(ViewTrainingProto::new(
                        self.id,
                        self.preset.clone(),
                        self.go_back.take(),
                    )),
                );
                return Ok(Some(view));
            }
            TrainingProtoCallback::Back => {
                if let Some(widget) = self.go_back.take() {
                    return Ok(Some(widget));
                }
            }
            TrainingProtoCallback::Description => {
                let training = ctx
                    .ledger
                    .get_training_by_id(self.id)
                    .await?
                    .ok_or_else(|| eyre::eyre!("Training not found"))?;
                ctx.send_msg(&training.description).await?;
                let id = ctx.send_msg("\\.").await?;
                ctx.update_origin_msg_id(id);
                self.show(ctx).await?;
            }
        }
        Ok(None)
    }
}

async fn render(
    ctx: &Context,
    training: &TrainingProto,
    go_back: bool,
) -> Result<(String, InlineKeyboardMarkup)> {
    let text = format!(
        "
🧘*Тренировка*: {}
*Продолжительность*: {}мин
*Вместимость*: {}
",
        escape(&training.name),
        training.duration_min,
        training.capacity
    );

    let mut keymap = Vec::new();
    keymap.push(vec![InlineKeyboardButton::callback(
        "📝Описание",
        TrainingProtoCallback::Description.to_data(),
    )]);

    if ctx.has_right(Rule::EditSchedule) {
        keymap.push(vec![InlineKeyboardButton::callback(
            "📅Запланировать",
            TrainingProtoCallback::Schedule.to_data(),
        )]);
    }

    if go_back {
        keymap.push(vec![InlineKeyboardButton::callback(
            "⬅️Назад",
            TrainingProtoCallback::Back.to_data(),
        )]);
    }

    Ok((text, InlineKeyboardMarkup::new(keymap)))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TrainingProtoCallback {
    Schedule,
    Description,
    Back,
}
