use super::{find_training::FindTraining, View};
use crate::{
    callback_data::Calldata as _,
    context::Context,
    state::Widget,
    view::calendar::{render_training_status, training::TrainingView, CallbackDateTime},
};
use async_trait::async_trait;
use eyre::Result;
use serde::{Deserialize, Serialize};
use storage::user::rights::Rule;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

#[derive(Default)]
pub struct MyTrainings {
    go_back: Option<Widget>,
}

impl MyTrainings {
    pub fn new(go_back: Option<Widget>) -> Self {
        Self { go_back }
    }
}

#[async_trait]
impl View for MyTrainings {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (msg, keyboard) = render(ctx, self.go_back.is_some()).await?;
        ctx.edit_origin(&msg, keyboard).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        ctx.delete_msg(message.id).await?;
        Ok(None)
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Option<Widget>> {
        match MyTrainingsCallback::from_data(data)? {
            MyTrainingsCallback::Back => Ok(self.go_back.take()),
            MyTrainingsCallback::SelectTraining(date) => {
                let widget = Box::new(TrainingView::new(
                    date.into(),
                    Some(Box::new(MyTrainings::new(self.go_back.take()))),
                ));
                Ok(Some(widget))
            }
            MyTrainingsCallback::FindTraining => {
                let this = Box::new(MyTrainings {
                    go_back: self.go_back.take(),
                });
                let widget = Box::new(FindTraining::new(Some(this)));
                Ok(Some(widget))
            }
        }
    }
}

async fn render(ctx: &Context, go_back: bool) -> Result<(String, InlineKeyboardMarkup)> {
    let mut msg = "🫶🏻 Мои тренировки:".to_owned();

    let mut keyboard = InlineKeyboardMarkup::default();

    let trainings = ctx
        .ledger
        .calendar
        .get_my_trainings(ctx.me.id, 100, 0)
        .await?;

    if trainings.is_empty() && !ctx.has_right(Rule::Train) {
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
🟢\\- запись открыта ⛔\\- тренировка отменена
🟠\\- запись закрыта ✔️\\- тренировка прошла
🔵\\- тренировка идет
➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖
",
    );

    for training in trainings[..std::cmp::min(15, trainings.len())].iter() {
        let mut row = vec![];
        row.push(InlineKeyboardButton::callback(
            format!(
                "{} {} {}",
                render_training_status(&training.status, training.is_full()),
                training.start_at.format("%d.%m %H:%M"),
                training.name.as_str(),
            ),
            MyTrainingsCallback::SelectTraining(training.start_at.into()).to_data(),
        ));
        keyboard = keyboard.append_row(row);
    }
    if !ctx.has_right(Rule::Train) {
        keyboard = keyboard.append_row(vec![InlineKeyboardButton::callback(
            "🔍 Найти тренировку",
            MyTrainingsCallback::FindTraining.to_data(),
        )]);
    }

    if go_back {
        keyboard = keyboard.append_row(vec![InlineKeyboardButton::callback(
            "🔙 Назад",
            MyTrainingsCallback::Back.to_data(),
        )]);
    }

    Ok((msg, keyboard))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MyTrainingsCallback {
    Back,
    SelectTraining(CallbackDateTime),
    FindTraining,
}
