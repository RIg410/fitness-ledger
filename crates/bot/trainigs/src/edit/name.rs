use async_trait::async_trait;
use bot_core::{
    context::Context,
    widget::{Jmp, View},
};
use eyre::{Ok, Result};
use model::{rights::Rule, training::TrainingId};
use teloxide::types::{InlineKeyboardMarkup, Message};

pub struct ChangeName {
    id: TrainingId,
}

impl ChangeName {
    pub fn new(id: TrainingId) -> ChangeName {
        ChangeName { id }
    }
}

#[async_trait]
impl View for ChangeName {
    fn name(&self) -> &'static str {
        "ChangeName"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::EditTraining)?;

        let msg = "Введите название тренировки";
        let keymap = InlineKeyboardMarkup::default();
        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, msg: &Message) -> Result<Jmp> {
        ctx.delete_msg(msg.id).await?;
        let name = msg.text().unwrap_or_default();
        if name.is_empty() {
            ctx.send_notification("Название не может быть пустым").await;
            return Ok(Jmp::Stay);
        }

        ctx.ledger
            .calendar
            .update_training_name(&mut ctx.session, self.id, name)
            .await?;
        ctx.send_notification("Название тренировки изменено").await;
        Ok(Jmp::BackSteps(3))
    }
}
