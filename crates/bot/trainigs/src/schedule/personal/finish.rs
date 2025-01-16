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
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

use super::{
    render_msg, set_date_time::render_time_slot_collision, PersonalTrainingPreset, DURATION,
};

#[derive(Default)]
pub struct Finish {
    preset: PersonalTrainingPreset,
}

impl Finish {
    pub fn new(preset: PersonalTrainingPreset) -> Self {
        Self { preset }
    }
}

#[async_trait]
impl View for Finish {
    fn name(&self) -> &'static str {
        "SchFinish"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let msg = render_msg(ctx, &self.preset, "Все верно?").await?;
        let keymap = vec![vec![
            Callback::Yes.button("✅ Сохранить"),
            Callback::No.button("❌ Отмена"),
        ]];
        ctx.edit_origin(&msg, InlineKeyboardMarkup::new(keymap))
            .await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::Yes => {
                ctx.ensure(Rule::EditSchedule)?;
                let preset = self.preset;
                let date_time = preset
                    .date_time
                    .ok_or_else(|| eyre::eyre!("DateTime is missing"))?;
                let instructor = preset
                    .instructor
                    .ok_or_else(|| eyre::eyre!("Instructor is missing"))?;
                let client = preset
                    .client
                    .ok_or_else(|| eyre::eyre!("Client is missing"))?;
                let room = preset.room.ok_or_else(|| eyre::eyre!("Room is missing"))?;

                match ctx
                    .ledger
                    .calendar
                    .schedule_personal_training(
                        &mut ctx.session,
                        client,
                        instructor,
                        date_time,
                        DURATION,
                        room,
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
        ScheduleError::ClientNotFound => "Клиент не найден".to_string(),
    }
}
