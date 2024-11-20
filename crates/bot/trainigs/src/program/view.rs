use crate::{list::TrainingList, schedule::ScheduleTrainingPreset};

use super::edit::EditProgram;
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::training::fmt_training_type;
use eyre::Result;
use model::{program::Program, rights::Rule};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

pub struct ProgramView {
    id: ObjectId,
    preset: ScheduleTrainingPreset,
}

impl ProgramView {
    pub fn new(id: ObjectId, preset: ScheduleTrainingPreset) -> Self {
        Self { id, preset }
    }

    async fn find_training(&mut self) -> Result<Jmp> {
        Ok(TrainingList::programs(self.id).into())
    }

    async fn schedule(&mut self, ctx: &mut Context) -> Result<Jmp> {
        ctx.ensure(Rule::EditSchedule)?;
        let preset = self.preset.clone();
        let view = preset.into_next_view(self.id);
        Ok(view.into())
    }

    async fn edit_capacity(&mut self, ctx: &mut Context) -> Result<Jmp> {
        ctx.ensure(Rule::EditTraining)?;
        Ok(EditProgram::new(self.id, super::edit::EditType::Capacity).into())
    }

    async fn edit_duration(&mut self, ctx: &mut Context) -> Result<Jmp> {
        ctx.ensure(Rule::EditTraining)?;
        Ok(EditProgram::new(self.id, super::edit::EditType::Duration).into())
    }

    async fn edit_name(&mut self, ctx: &mut Context) -> Result<Jmp> {
        ctx.ensure(Rule::EditTraining)?;
        Ok(EditProgram::new(self.id, super::edit::EditType::Name).into())
    }

    async fn edit_description(&mut self, ctx: &mut Context) -> Result<Jmp> {
        ctx.ensure(Rule::EditTraining)?;
        Ok(EditProgram::new(self.id, super::edit::EditType::Description).into())
    }

    async fn hide(&mut self, ctx: &mut Context, hide: bool) -> Result<Jmp> {
        ctx.ensure(Rule::EditTraining)?;

        ctx.ledger
            .programs
            .set_visible(&mut ctx.session, &self.id, !hide)
            .await?;

        Ok(Jmp::Stay)
    }
}

#[async_trait]
impl View for ProgramView {
    fn name(&self) -> &'static str {
        "ViewProgram"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let training = ctx
            .ledger
            .programs
            .get_by_id(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let (text, keymap) = render(ctx, &training).await?;
        ctx.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::Schedule => self.schedule(ctx).await,
            Callback::FindTraining => self.find_training().await,
            Callback::EditCapacity => self.edit_capacity(ctx).await,
            Callback::EditDuration => self.edit_duration(ctx).await,
            Callback::EditName => self.edit_name(ctx).await,
            Callback::EditDescription => self.edit_description(ctx).await,
            Callback::Hide(visible) => self.hide(ctx, visible).await,
        }
    }
}

async fn render(ctx: &Context, training: &Program) -> Result<(String, InlineKeyboardMarkup)> {
    let text = format!(
        "
🧘*Тренировка*: {}
*Продолжительность*: {}мин
*Вместимость*: {}
[Описание]({})
{}
",
        escape(&training.name),
        training.duration_min,
        training.capacity,
        escape(&training.description),
        fmt_training_type(training.tp),
    );

    let mut keymap = Vec::new();
    if ctx.has_right(Rule::EditSchedule) {
        keymap.push(vec![Callback::Schedule.button("📅Запланировать")]);
    }

    if ctx.has_right(Rule::EditTraining) {
        keymap.push(vec![
            Callback::EditDuration.button("🕤Изменить продолжительность")
        ]);
        keymap.push(vec![Callback::EditCapacity.button("👥Изменить вместимость")]);
        keymap.push(vec![Callback::EditName.button("📝Изменить название")]);
        keymap.push(vec![Callback::EditDescription.button("📝Изменить описание")]);

        if training.visible {
            keymap.push(vec![Callback::Hide(true).button("🔒Скрыть")]);
        } else {
            keymap.push(vec![Callback::Hide(false).button("🔓Показать")]);
        }
    }

    if !ctx.me.is_couch() {
        keymap.push(vec![Callback::FindTraining.button("📅Расписание")]);
    }

    Ok((text, InlineKeyboardMarkup::new(keymap)))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    Schedule,
    FindTraining,
    EditDuration,
    EditCapacity,
    EditName,
    EditDescription,
    Hide(bool),
}
