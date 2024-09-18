use crate::{
    callback_data::Calldata as _,
    context::Context,
    state::Widget,
    view::{
        training::{
            create_training::CreateTraining, schedule_process::ScheduleTrainingPreset,
            view_training_proto::ViewProgram,
        },
        View,
    },
};
use async_trait::async_trait;
use eyre::{Error, Result};
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::Requester as _,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

pub struct ProgramList {
    go_back: Option<Widget>,
}

impl ProgramList {
    pub fn new(go_back: Option<Widget>) -> Self {
        Self { go_back }
    }
}

#[async_trait]
impl View for ProgramList {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (msg, keymap) = render(ctx, self.go_back.is_some()).await?;
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
        let cb = if let Some(cb) = Callback::from_data(data) {
            cb
        } else {
            return Ok(None);
        };
        match cb {
            Callback::Back => {
                if let Some(widget) = self.go_back.take() {
                    return Ok(Some(widget));
                }
            }
            Callback::CreateTraining => {
                ctx.ensure(Rule::CreateTraining)?;
                return Ok(Some(CreateTraining::new(self.take()).boxed()));
            }
            Callback::SelectTraining(id) => {
                let id = ObjectId::from_bytes(id);
                let preset = ScheduleTrainingPreset {
                    day: None,
                    date_time: None,
                    instructor: None,
                    is_one_time: None,
                };
                return Ok(Some(
                    ViewProgram::new(id, preset, Some(self.take())).boxed(),
                ));
            }
        }
        Ok(None)
    }

    fn take(&mut self) -> Widget {
        ProgramList {
            go_back: self.go_back.take(),
        }
        .boxed()
    }
}

async fn render(
    ctx: &mut Context,
    has_back: bool,
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let msg = format!("Тренировочные программы: 🤸🏼");
    let mut keymap = InlineKeyboardMarkup::default();

    let trainings = ctx.ledger.programs.find(&mut ctx.session, None).await?;

    for training in trainings {
        keymap
            .inline_keyboard
            .push(Callback::SelectTraining(training.id.bytes()).btn_row(training.name));
    }

    if ctx.has_right(Rule::CreateTraining) {
        keymap
            .inline_keyboard
            .push(Callback::CreateTraining.btn_row("🧘🏼 Создать новую тренировку"));
    }
    if has_back {
        keymap
            .inline_keyboard
            .push(vec![InlineKeyboardButton::callback(
                "⬅️ Назад",
                Callback::Back.to_data(),
            )]);
    }
    Ok((msg, keymap))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    Back,
    CreateTraining,
    SelectTraining([u8; 12]),
}
