use crate::{callback_data::Calldata as _, context::Context, state::Widget, view::View};
use async_trait::async_trait;
use eyre::Result;
use model::{proto::TrainingProto, rights::Rule};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

pub struct CreateTraining {
    go_back: Option<Widget>,
    state: Option<State>,
}

impl CreateTraining {
    pub fn new(go_back: Widget) -> Self {
        Self {
            go_back: Some(go_back),
            state: None,
        }
    }
}

#[async_trait]
impl View for CreateTraining {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::CreateTraining)?;

        let keymap = if self.go_back.is_some() {
            vec![vec![InlineKeyboardButton::callback(
                "✖️ Отмена",
                Callback::Back.to_data(),
            )]]
        } else {
            vec![]
        };

        ctx.edit_origin(
            "📝 Введите название тренировки:\n_оно должно быть уникально_",
            InlineKeyboardMarkup::default().inline_keyboard(keymap),
        )
        .await?;
        self.state = Some(State::SetName(TrainingProto::default()));
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        ctx.ensure(Rule::CreateTraining)?;
        let msg = if let Some(msg) = message.text() {
            msg
        } else {
            return Ok(None);
        };

        let state = self
            .state
            .take()
            .ok_or_else(|| eyre::eyre!("State is missing"))?;
        self.state = Some(match state {
            State::SetName(mut training) => {
                if ctx.ledger.get_training_by_name(msg).await?.is_some() {
                    ctx.send_msg("Тренировка с таким названием уже существует")
                        .await?;
                    State::SetName(training)
                } else {
                    training.name = msg.to_string();
                    ctx.send_msg("📝 Введите описание тренировки").await?;
                    State::SetDescription(training)
                }
            }
            State::SetDescription(mut training) => {
                training.description = msg.to_string();
                ctx.send_msg("📝 Введите продолжительность тренировки в минутах")
                    .await?;
                State::SetDuration(training)
            }
            State::SetDuration(mut training) => {
                if let Ok(duration) = msg.parse::<u32>() {
                    training.duration_min = duration;
                    ctx.send_msg("📝 Введите количество мест на тренировке")
                        .await?;
                    State::SetCapacity(training)
                } else {
                    ctx.send_msg("Продолжительность должна быть числом").await?;
                    State::SetDuration(training)
                }
            }
            State::SetCapacity(mut training) => {
                if let Ok(capacity) = msg.parse::<u32>() {
                    training.capacity = capacity;
                    ctx.ensure(Rule::CreateTraining)?;
                    ctx.ledger.create_training_proto(&training).await?;
                    ctx.send_msg("✅ Тренировка создана").await?;
                    let origin = ctx.send_msg("\\.").await?;
                    ctx.update_origin_msg_id(origin);
                    return Ok(self.go_back.take());
                } else {
                    ctx.send_msg("Количество мест должно быть числом").await?;
                    State::SetCapacity(training)
                }
            }
        });
        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        match Callback::from_data(data)? {
            Callback::Back => {
                if let Some(widget) = self.go_back.take() {
                    let origin = ctx.send_msg("\\.").await?;
                    ctx.update_origin_msg_id(origin);
                    return Ok(Some(widget));
                }
            }
        }

        Ok(None)
    }
}

pub enum State {
    SetName(TrainingProto),
    SetDescription(TrainingProto),
    SetDuration(TrainingProto),
    SetCapacity(TrainingProto),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    Back,
}
