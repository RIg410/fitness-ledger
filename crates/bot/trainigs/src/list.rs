use async_trait::async_trait;
use bot_core::{
    callback_data::{CallbackDateTime, Calldata as _},
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::{
    day::{fmt_dt, fmt_weekday},
    training::fmt_training_status,
};
use chrono::Local;
use eyre::Result;
use model::training::Filter;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

use crate::view::TrainingView;

const TRAININGS_PER_PAGE: u32 = 7;

pub struct TrainingList {
    filter: Filter,
    offset: u32,
}

impl TrainingList {
    pub fn users(id: ObjectId) -> Self {
        Self {
            filter: Filter::Client(id),
            offset: 0,
        }
    }

    pub fn couches(id: ObjectId) -> Self {
        Self {
            filter: Filter::Instructor(id),
            offset: 0,
        }
    }

    pub fn programs(id: ObjectId) -> Self {
        Self {
            filter: Filter::Program(id),
            offset: 0,
        }
    }
}

#[async_trait]
impl View for TrainingList {
    fn name(&self) -> &'static str {
        "TrainingList"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (msg, keyboard) = render(ctx, self.filter.clone(), self.offset).await?;
        ctx.edit_origin(&msg, keyboard).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::SelectTraining(date) => Ok(TrainingView::new(date.into()).into()),
            Callback::Offset(offset) => {
                self.offset = offset;
                Ok(Jmp::None)
            }
        }
    }
}

async fn render(
    ctx: &mut Context,
    filter: Filter,
    offset: u32,
) -> Result<(String, InlineKeyboardMarkup)> {
    let mut msg = "🫶🏻 Мои тренировки:".to_owned();

    let mut keymap = InlineKeyboardMarkup::default();

    let trainings = ctx
        .ledger
        .calendar
        .find_trainings(
            &mut ctx.session,
            filter,
            TRAININGS_PER_PAGE as usize,
            offset as usize,
        )
        .await?;

    if trainings.is_empty() && ctx.me.couch.is_none() {
        msg.push_str("\n🤷🏻‍♂️  У вас нет назначенных тренировок");
    }

    msg.push_str("\n");
    msg.push_str(
        "
➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖
🟢\\- запись открыта 
⛔\\- тренировка отменена
🟠\\- запись закрыта 
✔️\\- тренировка прошла
🔵\\- тренировка идет
➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖
",
    );

    let now = Local::now();
    for training in trainings.iter() {
        let mut row = vec![];
        let slot = training.get_slot();
        let start_at = slot.start_at();
        row.push(Callback::SelectTraining(start_at.into()).button(format!(
            "{} {} {} {}",
            fmt_training_status(
                training.status(now),
                training.is_processed,
                training.is_full(),
                training.clients.contains(&ctx.me.id)
            ),
            fmt_weekday(&start_at),
            start_at.format("%d.%m %H:%M"),
            training.name.as_str(),
        )));
        keymap = keymap.append_row(row);
    }
    let mut row = vec![];

    if offset > 0 {
        row.push(Callback::Offset(offset - TRAININGS_PER_PAGE).button("⬅️"));
    }
    if (trainings.len() as u32) >= TRAININGS_PER_PAGE {
        row.push(Callback::Offset(offset + TRAININGS_PER_PAGE).button("➡️"));
    };
    keymap = keymap.append_row(row);

    Ok((msg, keymap))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    SelectTraining(CallbackDateTime),
    Offset(u32),
}
