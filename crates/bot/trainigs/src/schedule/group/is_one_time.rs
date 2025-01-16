use super::{render_msg, ScheduleTrainingPreset};
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

#[derive(Default)]
pub struct SetOneTime {
    preset: ScheduleTrainingPreset,
}

impl SetOneTime {
    pub fn new(preset: ScheduleTrainingPreset) -> Self {
        Self { preset }
    }
}

#[async_trait]
impl View for SetOneTime {
    fn name(&self) -> &'static str {
        "SetOneTime"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let training = ctx
            .ledger
            .programs
            .get_by_id(&mut ctx.session, self.preset.program_id()?)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let msg = render_msg(
            ctx,
            &training,
            &self.preset,
            "Это разовая тренировка или регулярная?",
        )
        .await?;
        let mut keymap = InlineKeyboardMarkup::default();
        keymap.inline_keyboard.push(vec![
            Callback::OneTime.button("разовая"),
            Callback::Regular.button("регулярная"),
        ]);
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::OneTime => {
                self.preset.is_one_time = Some(true);
            }
            Callback::Regular => {
                self.preset.is_one_time = Some(false);
            }
        };
        Ok(self.preset.into_next_view().into())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Callback {
    OneTime,
    Regular,
}
