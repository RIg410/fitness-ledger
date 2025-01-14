use super::{render_msg, set_date_time::render_time_slot_collision, ScheduleTrainingPreset};
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::Result;
use ledger::service::calendar::ScheduleError;
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

#[derive(Default)]
pub struct Finish {
    id: ObjectId,
    preset: Option<ScheduleTrainingPreset>,
}

impl Finish {
    pub fn new(id: ObjectId, preset: ScheduleTrainingPreset) -> Self {
        Self {
            id,
            preset: Some(preset),
        }
    }
}

#[async_trait]
impl View for Finish {
    fn name(&self) -> &'static str {
        "SchFinish"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let training = ctx
            .ledger
            .programs
            .get_by_id(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let msg = render_msg(ctx, &training, self.preset.as_ref().unwrap()).await?;
        ctx.send_msg(&msg).await?;
        let msg = "Все верно?".to_string();
        let keymap = vec![vec![
            Callback::Yes.button("✅ Сохранить"),
            Callback::No.button("❌ Отмена"),
        ]];
        ctx.send_msg_with_markup(&msg, InlineKeyboardMarkup::new(keymap))
            .await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::Yes => {
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

                let room = preset.room.ok_or_else(|| eyre::eyre!("Room is missing"))?;

                match ctx
                    .ledger
                    .calendar
                    .schedule(
                        &mut ctx.session,
                        self.id,
                        date_time,
                        room,
                        instructor,
                        is_one_time,
                    )
                    .await
                {
                    Ok(_) => {
                        ctx.send_msg("Тренировка успешно добавлена ✅").await?;
                    }
                    Err(err) => {
                        ctx.send_msg(&error_msg(&err)).await?;
                    }
                }
            }
            Callback::No => {
                //no-op
            }
        }
        Ok(Jmp::BackSteps(8))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    Yes,
    No,
}

fn error_msg(err: &ScheduleError) -> String {
    match err {
        ScheduleError::ProgramNotFound => "Тренировка не найдена".to_string(),
        ScheduleError::InstructorNotFound => "Инструктор не найден".to_string(),
        ScheduleError::InstructorHasNoRights => {
            "Инструктор не имеет прав на проведение тренировок".to_string()
        }
        ScheduleError::TimeSlotCollision(collision) => render_time_slot_collision(collision),
        ScheduleError::Common(err) => escape(&format!("Ошибка: {:#}", err)),
        ScheduleError::TooCloseToStart => {
            "Нельзя добавить тренировку менее чем за 3 часа".to_string()
        }
    }
}
