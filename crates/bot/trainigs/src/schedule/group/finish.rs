use super::{render_msg, ScheduleTrainingPreset};
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::Result;
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

#[derive(Default)]
pub struct Finish {
    preset: ScheduleTrainingPreset,
}

impl Finish {
    pub fn new(preset: ScheduleTrainingPreset) -> Self {
        Self { preset }
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
            .get_by_id(&mut ctx.session, self.preset.program_id()?)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let msg = render_msg(ctx, &training, &self.preset, "Все верно?").await?;
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
                ctx.ensure(Rule::ScheduleGroupTraining)?;
                let preset = self.preset;
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

                ctx.ledger
                    .calendar
                    .schedule_group(
                        &mut ctx.session,
                        preset.program_id()?,
                        date_time,
                        room,
                        instructor,
                        is_one_time,
                    )
                    .await?;
                ctx.send_msg("Тренировка успешно добавлена ✅").await?;
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
