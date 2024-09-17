use super::{render_msg, ScheduleTrainingPreset};
use crate::{callback_data::Calldata as _, context::Context, state::Widget, view::View};
use async_trait::async_trait;
use eyre::Result;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::Requester as _,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

#[derive(Default)]
pub struct SetOneTime {
    id: ObjectId,
    preset: Option<ScheduleTrainingPreset>,
    go_back: Option<Widget>,
}

impl SetOneTime {
    pub fn new(id: ObjectId, preset: ScheduleTrainingPreset, go_back: Widget) -> Self {
        Self {
            id,
            preset: Some(preset),
            go_back: Some(go_back),
        }
    }
}

#[async_trait]
impl View for SetOneTime {
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
            InlineKeyboardButton::callback("разовая", Callback::OneTime.to_data()),
            InlineKeyboardButton::callback("регулярная", Callback::Regular.to_data()),
        ]);
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

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Option<Widget>> {
        let cb = if let Some(cb) = Callback::from_data(data) {
            cb
        } else {
            return Ok(None);
        };
        match cb {
            Callback::OneTime => {
                self.preset.as_mut().unwrap().is_one_time = Some(true);
            }
            Callback::Regular => {
                self.preset.as_mut().unwrap().is_one_time = Some(false);
            }
        };
        let preset = self.preset.take().unwrap();
        Ok(Some(
            preset.into_next_view(self.id, self.go_back.take().unwrap()),
        ))
    }
    fn take(&mut self) -> Widget {
        SetOneTime {
            id: self.id,
            preset: self.preset.take(),
            go_back: self.go_back.take(),
        }
        .boxed()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Callback {
    OneTime,
    Regular,
}
