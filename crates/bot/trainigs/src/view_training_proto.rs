use super::{edit::EditProgram, schedule_process::ScheduleTrainingPreset};
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Goto, View},
};
use eyre::Result;
use model::{ids::WeekId, program::Program, rights::Rule};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::Requester as _,
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct ViewProgram {
    id: ObjectId,
    preset: ScheduleTrainingPreset,
}

impl ViewProgram {
    pub fn new(id: ObjectId, preset: ScheduleTrainingPreset) -> Self {
        Self { id, preset }
    }

    async fn find_training(&mut self) -> Result<Goto> {
        let view = CalendarView::new(
            WeekId::default(),
            None,
            Some(Filter {
                proto_id: Some(self.id),
            }),
        );
        return Ok(Some(Box::new(view)));
    }

    async fn schedule(&mut self, ctx: &mut Context) -> Result<Goto> {
        ctx.ensure(Rule::EditSchedule)?;
        let preset = self.preset.clone();
        let view = preset.into_next_view(self.id);
        Ok(view.into())
    }

    async fn edit_capacity(&mut self, ctx: &mut Context) -> Result<Goto> {
        ctx.ensure(Rule::EditTraining)?;
        Ok(EditProgram::new(self.id, super::edit::EditType::Capacity).into())
    }

    async fn edit_duration(&mut self, ctx: &mut Context) -> Result<Goto> {
        ctx.ensure(Rule::EditTraining)?;
        Ok(EditProgram::new(self.id, super::edit::EditType::Duration).into())
    }

    async fn edit_name(&mut self, ctx: &mut Context) -> Result<Goto> {
        ctx.ensure(Rule::EditTraining)?;
        Ok(EditProgram::new(self.id, super::edit::EditType::Name).into())
    }

    async fn edit_description(&mut self, ctx: &mut Context) -> Result<Goto> {
        ctx.ensure(Rule::EditTraining)?;
        Ok(EditProgram::new(self.id, super::edit::EditType::Description).into())
    }
}

#[async_trait]
impl View for ViewProgram {
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

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Goto> {
        match calldata!(data) {
            Callback::Schedule => self.schedule(ctx).await,
            Callback::FindTraining => self.find_training().await,
            Callback::EditCapacity => self.edit_capacity(ctx).await,
            Callback::EditDuration => self.edit_duration(ctx).await,
            Callback::EditName => self.edit_name(ctx).await,
            Callback::EditDescription => self.edit_description(ctx).await,
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
",
        escape(&training.name),
        training.duration_min,
        training.capacity,
        escape(&training.description),
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
}
