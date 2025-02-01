use crate::schedule::group::set_date_time::TimeParts;
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use chrono::{Local, Timelike, Utc};
use eyre::Result;
use log::warn;
use model::{rights::Rule, slot::Slot, training::TrainingId};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};

pub struct ChangeTime {
    id: TrainingId,
    all: bool,
}

impl ChangeTime {
    pub fn new(id: TrainingId, all: bool) -> ChangeTime {
        ChangeTime { id, all }
    }
}

#[async_trait]
impl View for ChangeTime {
    fn name(&self) -> &'static str {
        "ChangeTime"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::ChangeTrainingSlot)?;

        let msg = "Введите время начала тренировки в формате HH:MM";
        let keymap = InlineKeyboardMarkup::default();
        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, msg: &Message) -> Result<Jmp> {
        ctx.delete_msg(msg.id).await?;
        let msg = if let Some(msg) = msg.text() {
            msg
        } else {
            return Ok(Jmp::Stay);
        };

        let parts = match TimeParts::try_from(msg) {
            Ok(parts) => parts,
            Err(err) => {
                warn!("Invalid time format: {}", err);
                ctx.send_msg("Неверный формат времени\\.").await?;
                return Ok(Jmp::Stay);
            }
        };
        let hours = parts.0;
        let minute = parts.1;

        let start_at = if let Some(start_at) = self
            .id
            .start_at
            .with_timezone(&Local)
            .with_hour(hours)
            .and_then(|t| t.with_minute(minute))
        {
            start_at.with_timezone(&Utc)
        } else {
            ctx.send_msg("Неверный формат времени\\.").await?;
            return Ok(Jmp::Stay);
        };

        let training = ctx
            .ledger
            .calendar
            .get_training_by_id(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;

        Ok(ConfirmChangeTime::new(
            self.id,
            Slot::new(start_at, training.duration_min, self.id.room),
            self.all,
        )
        .into())
    }
}

pub struct ConfirmChangeTime {
    id: TrainingId,
    slot: Slot,
    all: bool,
}
impl ConfirmChangeTime {
    pub fn new(id: TrainingId, slot: Slot, all: bool) -> ConfirmChangeTime {
        ConfirmChangeTime { id, slot, all }
    }
}

#[async_trait]
impl View for ConfirmChangeTime {
    fn name(&self) -> &'static str {
        "ConfirmChangeTime"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::ChangeTrainingSlot)?;

        let msg = if self.all {
            format!(
                "Изменить время тренировок с {} на {}?",
                self.id.start_at.with_timezone(&Local).format("%H:%M"),
                self.slot.start_at().format("%H:%M")
            )
        } else {
            format!(
                "Изменить время тренировки с {} на {}?",
                self.id.start_at.with_timezone(&Local).format("%H:%M"),
                self.slot.start_at().format("%H:%M")
            )
        };

        let mut keymap = InlineKeyboardMarkup::default();
        keymap = keymap.append_row(vec![
            ConfirmCallback::Confirm.button("✅ Подтвердить"),
            ConfirmCallback::Cancel.button("❌ Отмена"),
        ]);
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            ConfirmCallback::Confirm => {
                ctx.ensure(Rule::ChangeTrainingSlot)?;

                ctx.ledger
                    .calendar
                    .change_slot(&mut ctx.session, self.id, self.slot, self.all)
                    .await?;
                ctx.send_notification("Время тренировки изменено").await;
                Ok(Jmp::BackSteps(4))
            }
            ConfirmCallback::Cancel => Ok(Jmp::BackSteps(2)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum ConfirmCallback {
    Confirm,
    Cancel,
}
