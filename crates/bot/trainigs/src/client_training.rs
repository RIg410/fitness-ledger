use super::find_training::FindTraining;
use async_trait::async_trait;
use bot_core::{
    callback_data::{CallbackDateTime, Calldata as _},
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::training::fmt_training_status;
use chrono::Local;
use eyre::Result;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

pub struct ClientTrainings {
    id: ObjectId,
}

impl ClientTrainings {
    pub fn new(id: ObjectId) -> Self {
        Self { id }
    }
}

#[async_trait]
impl View for ClientTrainings {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (msg, keyboard) = render(ctx, self.id).await?;
        ctx.edit_origin(&msg, keyboard).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::SelectTraining(date) => {
                // let widget = Box::new(TrainingView::new(date.into()));
                // Ok(Some(widget))
                todo!()
            }
            Callback::FindTraining => Ok(FindTraining::default().into()),
        }
    }
}

async fn render(ctx: &mut Context, id: ObjectId) -> Result<(String, InlineKeyboardMarkup)> {
    let mut msg = "🫶🏻 Мои тренировки:".to_owned();

    let mut keymap = InlineKeyboardMarkup::default();

    let trainings = ctx
        .ledger
        .calendar
        .get_users_trainings(&mut ctx.session, id, 100, 0)
        .await?;

    if trainings.is_empty() && ctx.me.couch.is_none() {
        msg.push_str("\n\n🤷🏻‍♂️  У вас нет назначенных тренировок");
        msg.push_str("\n\n🔍давайте что\\-нибудь подберем");
    }
    if trainings.len() > 0 {
        msg.push_str(format!("\n\n _{}_\\- тренировок", trainings.len()).as_str());
    }

    msg.push_str("\n\n");
    msg.push_str(
        "
➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖
🟢\\- запись открыта 
⛔\\- тренировка отменена
🟠\\- запись закрыта 
✔️\\- тренировка прошла
🔵\\- тренировка идет
➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖
",
    );

    let now = Local::now();
    for training in trainings[..std::cmp::min(15, trainings.len())].iter() {
        let mut row = vec![];
        let slot = training.get_slot();
        row.push(
            Callback::SelectTraining(slot.start_at().into()).button(format!(
                "{} {} {}",
                fmt_training_status(
                    training.status(now),
                    training.is_processed,
                    training.is_full(),
                    training.clients.contains(&ctx.me.id)
                ),
                slot.start_at().format("%d.%m %H:%M"),
                training.name.as_str(),
            )),
        );
        keymap = keymap.append_row(row);
    }
    if !ctx.is_couch() {
        keymap = keymap.append_row(Callback::FindTraining.btn_row("🔍 Найти тренировку"));
    }

    Ok((msg, keymap))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    SelectTraining(CallbackDateTime),
    FindTraining,
}
