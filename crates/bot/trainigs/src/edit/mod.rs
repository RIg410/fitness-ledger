use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::day::fmt_dt;
use change_couch::ChangeCouch;
use eyre::{bail, Result};
use model::{rights::Rule, training::TrainingId};
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

pub mod change_couch;
pub mod name;

pub struct EditTraining {
    id: TrainingId,
}

impl EditTraining {
    pub fn new(id: TrainingId) -> Self {
        Self { id }
    }

    pub fn hidden(ctx: &mut Context) -> Result<bool> {
        let show = ctx.has_right(Rule::EditTraining)
            || ctx.has_right(Rule::EditTrainingCouch)
            || ctx.has_right(Rule::RemoveTraining)
            || ctx.has_right(Rule::SetKeepOpen)
            || ctx.has_right(Rule::SetFree);
        Ok(!show)
    }

    async fn change_couch(&mut self, ctx: &mut Context, all: bool) -> Result<Jmp> {
        ctx.ensure(Rule::EditTrainingCouch)?;
        Ok(ChangeCouch::new(self.id, all).into())
    }

    async fn delete_training(&mut self, ctx: &mut Context, all: bool) -> Result<Jmp> {
        ctx.ensure(Rule::RemoveTraining)?;

        let training = ctx
            .ledger
            .calendar
            .get_training_by_id(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        if !training.is_group() {
            bail!("Can't delete personal training");
        }

        ctx.ledger
            .calendar
            .delete_training(&mut ctx.session, training.id(), all)
            .await?;
        Ok(Jmp::BackSteps(2))
    }

    async fn keep_open(&mut self, ctx: &mut Context, keep_open: bool) -> Result<Jmp> {
        ctx.ensure(Rule::SetKeepOpen)?;
        let training = ctx
            .ledger
            .calendar
            .get_training_by_id(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        if !training.is_group() {
            bail!("Can't delete personal training");
        }
        ctx.ledger
            .calendar
            .set_keep_open(&mut ctx.session, training.id(), keep_open)
            .await?;
        Ok(Jmp::Stay)
    }

    async fn set_free(&mut self, ctx: &mut Context, free: bool) -> Result<Jmp> {
        ctx.ensure(Rule::SetFree)?;

        let training = ctx
            .ledger
            .calendar
            .get_training_by_id(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        if !training.is_group() {
            bail!("Can't delete personal training");
        }
        if !training.clients.is_empty() {
            ctx.send_msg("Нельзя изменить статус тренировки, на которую записаны клиенты")
                .await?;
            return Ok(Jmp::Stay);
        }

        let mut tp = training.tp;
        tp.set_is_free(free);

        ctx.ledger
            .calendar
            .set_training_type(&mut ctx.session, training.id(), tp)
            .await?;
        Ok(Jmp::Stay)
    }
}

#[async_trait]
impl View for EditTraining {
    fn name(&self) -> &'static str {
        "EditTraining"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let training = ctx
            .ledger
            .calendar
            .get_training_by_id(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;

        let msg = format!(
            "Редактировать тренировку *{}*\nв _{}_",
            escape(&training.name),
            fmt_dt(&training.get_slot().start_at())
        );

        let mut keymap = InlineKeyboardMarkup::default();

        if ctx.has_right(Rule::SetKeepOpen) {
            if training.keep_open {
                keymap = keymap.append_row(vec![
                    Callback::KeepOpen(false).button("🔒 Закрыть для записи")
                ]);
            } else {
                keymap = keymap.append_row(vec![
                    Callback::KeepOpen(true).button("🔓 Открыть для записи")
                ]);
            }
        }
        if ctx.has_right(Rule::RemoveTraining) {
            keymap = keymap.append_row(vec![
                Callback::Delete(false).button("🗑️ Удалить эту тренировку")
            ]);
            if !training.is_one_time {
                keymap = keymap.append_row(vec![
                    Callback::Delete(true).button("🗑️ Удалить все последующие")
                ]);
            }
        }
        if ctx.has_right(Rule::EditTrainingCouch) {
            keymap = keymap.append_row(vec![
                Callback::ChangeCouch(false).button("🔄 Заменить инструктора")
            ]);
            keymap = keymap.append_row(vec![
                Callback::ChangeCouch(true).button("🔄 Заменить инструктора на все")
            ]);
        }

        if ctx.has_right(Rule::SetFree) {
            if training.tp.is_free() {
                keymap =
                    keymap.append_row(vec![Callback::SetFree(false).button("Сделать платной")]);
            } else {
                keymap =
                    keymap.append_row(vec![Callback::SetFree(true).button("Сделать бесплатной")]);
            }
        }

        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::ChangeCouch(all) => self.change_couch(ctx, all).await,
            Callback::Delete(all) => self.delete_training(ctx, all).await,
            Callback::KeepOpen(keep_open) => self.keep_open(ctx, keep_open).await,
            Callback::SetFree(free) => self.set_free(ctx, free).await,
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    ChangeCouch(bool),
    Delete(bool),
    KeepOpen(bool),
    SetFree(bool),
}
