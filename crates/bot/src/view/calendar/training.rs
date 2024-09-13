use super::View;
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use eyre::Result;
use model::{
    rights::Rule,
    training::{Training, TrainingStatus},
};
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::Requester as _,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct TrainingView {
    id: DateTime<Local>,
    go_back: Option<Widget>,
}

impl TrainingView {
    pub fn new(id: DateTime<Local>, go_back: Option<Widget>) -> Self {
        Self { id, go_back }
    }
}

#[async_trait]
impl View for TrainingView {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let (msg, keymap) = render(ctx, &training, self.go_back.is_some());
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
            Callback::Back => Ok(self.go_back.take()),
            Callback::Description => {
                let training = ctx
                    .ledger
                    .calendar
                    .get_training_by_start_at(&mut ctx.session, self.id)
                    .await?
                    .ok_or_else(|| eyre::eyre!("Training not found"))?;
                ctx.send_msg(&escape(&training.description)).await?;
                let id = ctx.send_msg("\\.").await?;
                ctx.update_origin_msg_id(id);
                self.show(ctx).await?;
                Ok(None)
            }
            Callback::Cancel => {
                ctx.ensure(Rule::CancelTraining)?;
                let training = ctx
                    .ledger
                    .calendar
                    .get_training_by_start_at(&mut ctx.session, self.id)
                    .await?
                    .ok_or_else(|| eyre::eyre!("Training not found"))?;
                ctx.ledger
                    .calendar
                    .cancel_training(&mut ctx.session, &training)
                    .await?;
                self.show(ctx).await?;
                Ok(None)
            }
            Callback::Delete(all) => {
                ctx.ensure(Rule::EditSchedule)?;
                let training = ctx
                    .ledger
                    .calendar
                    .get_training_by_start_at(&mut ctx.session, self.id)
                    .await?
                    .ok_or_else(|| eyre::eyre!("Training not found"))?;
                ctx.ledger
                    .calendar
                    .delete_training(&mut ctx.session, &training, all)
                    .await?;
                Ok(self.go_back.take())
            }
            Callback::UnCancel => {
                ctx.ensure(Rule::CancelTraining)?;
                let training = ctx
                    .ledger
                    .calendar
                    .get_training_by_start_at(&mut ctx.session, self.id)
                    .await?
                    .ok_or_else(|| eyre::eyre!("Training not found"))?;
                ctx.ledger
                    .calendar
                    .restore_training(&mut ctx.session, &training)
                    .await?;
                self.show(ctx).await?;
                Ok(None)
            }
            Callback::SignUp => {
                let training = ctx
                    .ledger
                    .calendar
                    .get_training_by_start_at(&mut ctx.session, self.id)
                    .await?
                    .ok_or_else(|| eyre::eyre!("Training not found"))?;
                if !training.status(Local::now()).can_sign_in() {
                    ctx.send_msg("Запись на тренировку закрыта💔").await?;
                    let id = ctx.send_msg("\\.").await?;
                    ctx.update_origin_msg_id(id);
                    self.show(ctx).await?;
                    return Ok(None);
                }

                if ctx.me.balance < 1 {
                    ctx.send_msg("Недостаточно средств на балансе🥺").await?;
                    let id = ctx.send_msg("\\.").await?;
                    ctx.update_origin_msg_id(id);
                    self.show(ctx).await?;
                    return Ok(None);
                }
                if ctx.me.freeze.is_some() {
                    ctx.send_msg("Ваш абонемент заморожен🥶").await?;
                    let id = ctx.send_msg("\\.").await?;
                    ctx.update_origin_msg_id(id);
                    self.show(ctx).await?;
                    return Ok(None);
                }

                ctx.ledger
                    .sign_up(&mut ctx.session, &training, ctx.me.id)
                    .await?;
                self.show(ctx).await?;
                Ok(None)
            }
            Callback::SignOut => {
                let training = ctx
                    .ledger
                    .calendar
                    .get_training_by_start_at(&mut ctx.session, self.id)
                    .await?
                    .ok_or_else(|| eyre::eyre!("Training not found"))?;
                if !training.status(Local::now()).can_sign_out() {
                    ctx.send_msg("Запись на тренировку закрыта").await?;
                    let id = ctx.send_msg("\\.").await?;
                    ctx.update_origin_msg_id(id);
                    self.show(ctx).await?;
                    return Ok(None);
                }
                ctx.ledger
                    .sign_out(&mut ctx.session, &training, ctx.me.id)
                    .await?;
                self.show(ctx).await?;
                Ok(None)
            }
            Callback::ClientList => {
                ctx.ensure(Rule::Train)?;
                let mut msg = "*Список клиентов:*\n".to_string();
                let training = ctx
                    .ledger
                    .calendar
                    .get_training_by_start_at(&mut ctx.session, self.id)
                    .await?
                    .ok_or_else(|| eyre::eyre!("Training not found"))?;
                for client in &training.clients {
                    let user = ctx
                        .ledger
                        .users
                        .get(&mut ctx.session, *client)
                        .await?
                        .ok_or_else(|| eyre::eyre!("User not found"))?;
                    msg.push_str(&format!(
                        "✅_{}_ _{}_{}\n",
                        escape(&user.name.first_name),
                        escape(&user.name.last_name.unwrap_or_else(|| "-".to_string())),
                        escape(
                            &user
                                .name
                                .tg_user_name
                                .map(|n| format!("@{}", n))
                                .unwrap_or_else(|| "".to_string())
                        )
                    ));
                }
                ctx.send_msg(&msg).await?;
                let id = ctx.send_msg("\\.").await?;
                ctx.update_origin_msg_id(id);
                self.show(ctx).await?;
                Ok(None)
            }
        }
    }
}

fn render(ctx: &Context, training: &Training, has_back: bool) -> (String, InlineKeyboardMarkup) {
    let is_client = !ctx.has_right(Rule::Train);
    let cap = if is_client {
        format!(
            "*свободных мест*: _{}_",
            training.capacity - training.clients.len() as u32
        )
    } else {
        format!(
            "*места* :_{}/{}_",
            training.clients.len(),
            training.capacity
        )
    };

    let now = Local::now();
    let tr_status = training.status(now);
    let slot = training.get_slot();

    let msg = format!(
        "
💪 *Тренировка*: _{}_
📅 *Дата*: _{}_
💁{}
_{}_
",
        escape(&training.name),
        slot.start_at().format("%d\\.%m\\.%Y %H:%M"),
        cap,
        status(tr_status, training.is_full()),
    );
    let mut keymap = InlineKeyboardMarkup::default();
    keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
        "📝 Описание",
        Callback::Description.to_data(),
    )]);

    if !is_client {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "🗒 Список клиентов",
            Callback::ClientList.to_data(),
        )]);
    }

    if ctx.has_right(Rule::CancelTraining) {
        if tr_status.can_be_canceled() {
            keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
                "⛔ Отменить",
                Callback::Cancel.to_data(),
            )]);
        }
        if tr_status.can_be_uncanceled() {
            keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
                "🔓 Вернуть",
                Callback::UnCancel.to_data(),
            )]);
        }
    }

    if ctx.has_right(Rule::EditSchedule) {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "🗑️ Удалить эту тренировку",
            Callback::Delete(false).to_data(),
        )]);
        if !training.is_one_time {
            keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
                "🗑️ Удалить все последующие",
                Callback::Delete(true).to_data(),
            )]);
        }
    }

    if is_client {
        if training.clients.contains(&ctx.me.id) {
            if tr_status.can_sign_out() {
                keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
                    "🔓 Отменить запись",
                    Callback::SignOut.to_data(),
                )]);
            }
        } else {
            if tr_status.can_sign_in() {
                keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
                    "🔒 Записаться",
                    Callback::SignUp.to_data(),
                )]);
            }
        }
    }
    if has_back {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "🔙 Назад",
            Callback::Back.to_data(),
        )]);
    }
    (msg, keymap)
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Back,
    Description,
    Delete(bool),
    Cancel,
    ClientList,
    UnCancel,
    SignUp,
    SignOut,
}

fn status(status: TrainingStatus, is_full: bool) -> &'static str {
    match status {
        TrainingStatus::OpenToSignup { .. } => {
            if is_full {
                "нет мест ✌️"
            } else {
                "🟢Открыта для записи"
            }
        }
        TrainingStatus::ClosedToSignup => "🟠Запись закрыта",
        TrainingStatus::InProgress => "🤸🏼 Идет",
        TrainingStatus::Cancelled => "⛔Отменена",
        TrainingStatus::Finished => "✔️Завершена",
    }
}
