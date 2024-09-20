use super::{render_msg, ScheduleTrainingPreset};
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::Result;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

#[derive(Default)]
pub struct SetOneTime {
    id: ObjectId,
    preset: Option<ScheduleTrainingPreset>,
}

impl SetOneTime {
    pub fn new(id: ObjectId, preset: ScheduleTrainingPreset) -> Self {
        Self {
            id,
            preset: Some(preset),
        }
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
            .get_by_id(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let msg = render_msg(ctx, &training, self.preset.as_ref().unwrap()).await?;
        ctx.send_msg(&msg).await?;
        let msg = "Это разовая тренировка или регулярная?".to_string();
        let mut keymap = InlineKeyboardMarkup::default();
        keymap.inline_keyboard.push(vec![
            Callback::OneTime.button("разовая"),
            Callback::Regular.button("регулярная"),
        ]);
        ctx.send_msg_with_markup(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::OneTime => {
                self.preset.as_mut().unwrap().is_one_time = Some(true);
            }
            Callback::Regular => {
                self.preset.as_mut().unwrap().is_one_time = Some(false);
            }
        };
        let preset = self.preset.take().unwrap();
        Ok(preset.into_next_view(self.id).into())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Callback {
    OneTime,
    Regular,
}
