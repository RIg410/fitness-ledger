use super::{
    create_training::CreateTraining, schedule_process::ScheduleTrainingPreset,
    view_training_proto::ViewTrainingProto,
};
use crate::{callback_data::Calldata as _, context::Context, state::Widget, view::{calendar::render_weekday, View}};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use eyre::{Error, Result};
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::Requester as _,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

pub struct ScheduleTraining {
    go_back: Option<Widget>,
    day: DateTime<Local>,
}

impl ScheduleTraining {
    pub fn new(day: DateTime<Local>, go_back: Option<Widget>) -> Self {
        Self { day, go_back }
    }
}

#[async_trait]
impl View for ScheduleTraining {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::EditSchedule)?;
        let (msg, keymap) = render(ctx, &self.day, self.go_back.is_some()).await?;
        ctx.edit_origin(&msg, keymap).await?;
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
        ctx.ensure(Rule::EditSchedule)?;
        match Callback::from_data(data)? {
            Callback::Back => {
                if let Some(widget) = self.go_back.take() {
                    return Ok(Some(widget));
                }
            }
            Callback::CreateTraining => {
                ctx.ensure(Rule::CreateTraining)?;
                let widget = Box::new(ScheduleTraining::new(self.day, self.go_back.take()));
                return Ok(Some(Box::new(CreateTraining::new(widget))));
            }
            Callback::SelectTraining(id) => {
                ctx.ensure(Rule::EditSchedule)?;
                let id = ObjectId::from_bytes(id);
                let widget = Box::new(ScheduleTraining::new(self.day, self.go_back.take()));

                let preset = ScheduleTrainingPreset {
                    day: Some(self.day),
                    date_time: None,
                    instructor: None,
                    is_one_time: None,
                };
                return Ok(Some(Box::new(ViewTrainingProto::new(
                    id,
                    preset,
                    Some(widget),
                ))));
            }
        }
        Ok(None)
    }
}

async fn render(
    ctx: &Context,
    day: &DateTime<Local>,
    has_back: bool,
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let msg = format!(
        "
🤸🏼 Добавить тренировку на день: *{}* _{}_
Выберите тренировку из списка или создайте новую\\.
",
        day.format("%d\\.%m\\.%Y"),
        render_weekday(day)
    );
    let mut markup = InlineKeyboardMarkup::default();

    let trainings = ctx.ledger.find_trainings(None).await?;

    for training in trainings {
        markup
            .inline_keyboard
            .push(vec![InlineKeyboardButton::callback(
                training.name.clone(),
                Callback::SelectTraining(training.id.bytes()).to_data(),
            )]);
    }

    markup
        .inline_keyboard
        .push(vec![InlineKeyboardButton::callback(
            "🧘🏼 Создать новую тренировку",
            Callback::CreateTraining.to_data(),
        )]);

    if has_back {
        markup
            .inline_keyboard
            .push(vec![InlineKeyboardButton::callback(
                "⬅️ Назад",
                Callback::Back.to_data(),
            )]);
    }
    Ok((msg, markup))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    Back,
    CreateTraining,
    SelectTraining([u8; 12]),
}
