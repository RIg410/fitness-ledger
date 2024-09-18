use super::{edit::EditProgram, schedule_process::ScheduleTrainingPreset};
use crate::{
    callback_data::Calldata as _,
    context::Context,
    state::Widget,
    view::{
        calendar::{CalendarView, Filter},
        View,
    },
};
use async_trait::async_trait;
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
    go_back: Option<Widget>,
    preset: ScheduleTrainingPreset,
}

impl ViewProgram {
    pub fn new(id: ObjectId, preset: ScheduleTrainingPreset, go_back: Option<Widget>) -> Self {
        Self {
            id,
            go_back,
            preset,
        }
    }

    async fn find_training(&mut self) -> Result<Option<Widget>> {
        let view = CalendarView::new(
            WeekId::default(),
            Some(self.take()),
            None,
            Some(Filter {
                proto_id: Some(self.id),
            }),
        );
        return Ok(Some(Box::new(view)));
    }

    async fn schedule(&mut self, ctx: &mut Context) -> Result<Option<Widget>> {
        ctx.ensure(Rule::EditSchedule)?;
        let preset = self.preset.clone();
        let view = preset.into_next_view(self.id, self.take());
        Ok(Some(view))
    }

    async fn edit_capacity(&mut self, ctx: &mut Context) -> Result<Option<Widget>> {
        ctx.ensure(Rule::EditTraining)?;
        Ok(Some(
            EditProgram::new(
                self.id,
                super::edit::EditType::Capacity,
                self.go_back.take(),
            )
            .boxed(),
        ))
    }

    async fn edit_duration(&mut self, ctx: &mut Context) -> Result<Option<Widget>> {
        ctx.ensure(Rule::EditTraining)?;
        Ok(Some(
            EditProgram::new(
                self.id,
                super::edit::EditType::Duration,
                self.go_back.take(),
            )
            .boxed(),
        ))
    }

    async fn edit_name(&mut self, ctx: &mut Context) -> Result<Option<Widget>> {
        ctx.ensure(Rule::EditTraining)?;
        Ok(Some(
            EditProgram::new(self.id, super::edit::EditType::Name, self.go_back.take()).boxed(),
        ))
    }

    async fn edit_description(&mut self, ctx: &mut Context) -> Result<Option<Widget>> {
        ctx.ensure(Rule::EditTraining)?;
        Ok(Some(
            EditProgram::new(
                self.id,
                super::edit::EditType::Description,
                self.go_back.take(),
            )
            .boxed(),
        ))
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
        let (text, keymap) = render(ctx, &training, self.go_back.is_some()).await?;
        ctx.edit_origin(&text, keymap).await?;
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

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        let cb = if let Some(cb) = Callback::from_data(data) {
            cb
        } else {
            return Ok(None);
        };

        match cb {
            Callback::Schedule => self.schedule(ctx).await,
            Callback::Back => {
                if let Some(widget) = self.go_back.take() {
                    Ok(Some(widget))
                } else {
                    Ok(None)
                }
            }
            Callback::FindTraining => self.find_training().await,
            Callback::EditCapacity => self.edit_capacity(ctx).await,
            Callback::EditDuration => self.edit_duration(ctx).await,
            Callback::EditName => self.edit_name(ctx).await,
            Callback::EditDescription => self.edit_description(ctx).await,
        }
    }

    fn take(&mut self) -> Widget {
        ViewProgram {
            id: self.id,
            go_back: self.go_back.take(),
            preset: self.preset.clone(),
        }
        .boxed()
    }
}

async fn render(
    ctx: &Context,
    training: &Program,
    go_back: bool,
) -> Result<(String, InlineKeyboardMarkup)> {
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

    if !ctx.has_right(Rule::Train) {
        keymap.push(vec![Callback::FindTraining.button("📅Расписание")]);
    }

    if go_back {
        keymap.push(vec![Callback::Back.button("⬅️Назад")]);
    }
    Ok((text, InlineKeyboardMarkup::new(keymap)))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    Schedule,
    Back,
    FindTraining,
    EditDuration,
    EditCapacity,
    EditName,
    EditDescription,
}
