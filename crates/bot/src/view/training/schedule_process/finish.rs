use super::{render_msg, ScheduleTrainingPreset, View};
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use ledger::training::AddTrainingError;
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::Requester as _,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

#[derive(Default)]
pub struct Finish {
    id: ObjectId,
    preset: Option<ScheduleTrainingPreset>,
    go_back: Option<Widget>,
}

impl Finish {
    pub fn new(id: ObjectId, preset: ScheduleTrainingPreset, go_back: Widget) -> Self {
        Self {
            id,
            preset: Some(preset),
            go_back: Some(go_back),
        }
    }
}

#[async_trait]
impl View for Finish {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let training = ctx
            .ledger
            .get_training_by_id(self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let msg = render_msg(ctx, &training, self.preset.as_ref().unwrap()).await?;
        ctx.send_msg(&msg).await?;
        let msg = format!("Все верно?");
        let keymap = vec![vec![
            InlineKeyboardButton::callback("✅ Сохранить", FinishCallback::Yes.to_data()),
            InlineKeyboardButton::callback("❌ Отмена", FinishCallback::No.to_data()),
        ]];
        ctx.send_msg_with_markup(&msg, InlineKeyboardMarkup::new(keymap))
            .await?;
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
        match FinishCallback::from_data(data)? {
            FinishCallback::Yes => {
                ctx.ensure(Rule::EditSchedule)?;
                let preset = self
                    .preset
                    .take()
                    .ok_or_else(|| eyre::eyre!("Preset is missing"))?;
                let date_time = preset
                    .date_time
                    .ok_or_else(|| eyre::eyre!("DateTime is missing"))?;
                let instructor = preset
                    .instructor
                    .ok_or_else(|| eyre::eyre!("Instructor is missing"))?;
                let is_one_time = preset
                    .is_one_time
                    .ok_or_else(|| eyre::eyre!("IsOneTime is missing"))?;

                match ctx
                    .ledger
                    .add_training(self.id, date_time, instructor, is_one_time)
                    .await
                {
                    Ok(_) => {
                        ctx.send_msg("Тренировка успешно добавлена ✅").await?;
                        let id = ctx.send_msg("\\.").await?;
                        ctx.update_origin_msg_id(id);
                    }
                    Err(err) => {
                        ctx.send_msg(&error_msg(&err)).await?;
                    }
                }
            }
            FinishCallback::No => {
                //no-op
            }
        }
        if let Some(widget) = self.go_back.take() {
            Ok(Some(widget))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FinishCallback {
    Yes,
    No,
}

fn error_msg(err: &AddTrainingError) -> String {
    match err {
        AddTrainingError::ProtoTrainingNotFound => "тренировка не найдена".to_string(),
        AddTrainingError::InstructorNotFound => "Инструктор не найден".to_string(),
        AddTrainingError::InstructorHasNoRights => {
            "Инструктор не имеет прав на проведение тренировок".to_string()
        }
        AddTrainingError::TimeSlotOccupied => "Время уже занято".to_string(),
        AddTrainingError::Common(err) => escape(&format!("Ошибка: {:#}", err)),
    }
}
