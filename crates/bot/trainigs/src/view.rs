use std::vec;

use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::{day::fmt_dt, fmt_phone, training::fmt_training_type};
use chrono::{DateTime, Local};
use eyre::{bail, Result};
use model::{
    rights::Rule,
    training::{Training, TrainingStatus},
};
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{ChatId, InlineKeyboardMarkup},
    utils::markdown::escape,
};

use crate::{change_couch::ChangeCouch, client::list::ClientsList};

pub struct TrainingView {
    id: DateTime<Local>,
}

impl TrainingView {
    pub fn new(id: DateTime<Local>) -> Self {
        Self { id }
    }

    async fn couch_info(&mut self, ctx: &mut Context) -> Result<Jmp> {
        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let user = ctx
            .ledger
            .get_user(&mut ctx.session, training.instructor)
            .await?;
        if let Some(couch) = user.couch {
            ctx.send_msg(&escape(&couch.description)).await?;
        }
        Ok(Jmp::Stay)
    }

    async fn delete_training(&mut self, ctx: &mut Context, all: bool) -> Result<Jmp> {
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
        Ok(Jmp::Back)
    }

    async fn restore_training(&mut self, ctx: &mut Context) -> Result<Jmp> {
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
        Ok(Jmp::Stay)
    }

    async fn sign_up(&mut self, ctx: &mut Context) -> Result<Jmp> {
        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        if !training.status(Local::now()).can_sign_in() {
            ctx.send_msg("Запись на тренировку закрыта💔").await?;
            return Ok(Jmp::Stay);
        }

        if training.is_full() {
            ctx.send_msg("Мест нет🥺").await?;
            return Ok(Jmp::Stay);
        }

        if let Some(sub) = ctx
            .me
            .find_subscription(model::user::FindFor::Lock, &training)
        {
            if sub.balance < 1 {
                ctx.send_msg("В абонементе нет занятий🥺").await?;
                return Ok(Jmp::Stay);
            }
        } else {
            ctx.send_msg("Нет подходящего абонемента🥺").await?;
            return Ok(Jmp::Stay);
        };

        if ctx.me.freeze.is_some() {
            ctx.send_msg("Ваш абонемент заморожен🥶").await?;
            return Ok(Jmp::Stay);
        }

        ctx.ledger
            .sign_up(&mut ctx.session, &training, ctx.me.id, false)
            .await?;

        let mut msg: String = format!(
            "Вы записались на тренировку '*{}*' в _{}_\\.\n",
            escape(&training.name),
            fmt_dt(&training.get_slot().start_at())
        );

        let balance = ctx.me.available_balance_for_training(&training);
        if balance <= 1 {
            msg.push_str("Ваш абонемент заканчивается🥺");
            if let Ok(users) = ctx
                .ledger
                .users
                .find_users_with_right(
                    &mut ctx.session,
                    Rule::ReceiveNotificationsAboutSubscriptions,
                )
                .await
            {
                for user in users {
                    let res = ctx
                        .bot
                        .send_notification_to(
                            ChatId(user.tg_id),
                            &format!(
                                "У {} {} заканчивается абонемент\\.",
                                escape(&ctx.me.name.to_string()),
                                fmt_phone(&ctx.me.phone)
                            ),
                        )
                        .await;
                    if let Err(e) = res {
                        log::error!("Failed to send notification to {}: {}", user.tg_id, e);
                    }
                }
            }
        }
        ctx.send_notification(&msg).await?;

        Ok(Jmp::Stay)
    }

    async fn sign_out(&mut self, ctx: &mut Context) -> Result<Jmp> {
        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        if !training.status(Local::now()).can_sign_out() {
            ctx.send_msg("Запись на тренировку закрыта").await?;
            return Ok(Jmp::Stay);
        }
        ctx.ledger
            .sign_out(&mut ctx.session, &training, ctx.me.id, false)
            .await?;
        Ok(Jmp::Stay)
    }

    async fn client_list(&mut self, ctx: &mut Context) -> Result<Jmp> {
        if !ctx.is_couch() && !ctx.has_right(Rule::EditTrainingClientsList) {
            bail!("Only couch can see client list");
        }
        Ok(ClientsList::new(self.id).into())
    }

    async fn change_couch(&mut self, ctx: &mut Context, all: bool) -> Result<Jmp> {
        ctx.ensure(Rule::EditSchedule)?;
        Ok(ChangeCouch::new(self.id, all).into())
    }

    async fn keep_open(&mut self, ctx: &mut Context, keep_open: bool) -> Result<Jmp> {
        ctx.ensure(Rule::SetKeepOpen)?;
        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        ctx.ledger
            .calendar
            .set_keep_open(&mut ctx.session, training.start_at, keep_open)
            .await?;
        Ok(Jmp::Stay)
    }

    async fn set_free(&mut self, ctx: &mut Context, free: bool) -> Result<Jmp> {
        ctx.ensure(Rule::SetFree)?;
        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;

        let mut tp = training.tp;
        tp.set_is_free(free);

        ctx.ledger
            .calendar
            .set_training_type(&mut ctx.session, training.start_at, tp)
            .await?;
        Ok(Jmp::Stay)
    }
}

#[async_trait]
impl View for TrainingView {
    fn name(&self) -> &'static str {
        "TrainingView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let (msg, keymap) = render(ctx, &training).await?;
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::CouchInfo => self.couch_info(ctx).await,
            Callback::Cancel => return Ok(Jmp::Next(ConfirmCancelTraining::new(self.id).into())),
            Callback::Delete(all) => self.delete_training(ctx, all).await,
            Callback::UnCancel => self.restore_training(ctx).await,
            Callback::SignUp => self.sign_up(ctx).await,
            Callback::SignOut => self.sign_out(ctx).await,
            Callback::ClientList => self.client_list(ctx).await,
            Callback::ChangeCouchOne => self.change_couch(ctx, false).await,
            Callback::ChangeCouchAll => self.change_couch(ctx, true).await,
            Callback::KeepOpen(keep_open) => self.keep_open(ctx, keep_open).await,
            Callback::SetFree(free) => self.set_free(ctx, free).await,
        }
    }
}

async fn render(ctx: &mut Context, training: &Training) -> Result<(String, InlineKeyboardMarkup)> {
    let is_client = ctx.me.couch.is_none();
    let cap = if is_client {
        format!(
            "*Свободных мест*: _{}_",
            training.capacity - training.clients.len() as u32
        )
    } else {
        format!(
            "*Места* :_{}/{}_",
            training.clients.len(),
            training.capacity
        )
    };

    let now = Local::now();
    let tr_status = training.status(now);
    let slot = training.get_slot();

    let couch = ctx
        .ledger
        .users
        .get(&mut ctx.session, training.instructor)
        .await?
        .map(|couch| {
            format!(
                "_{}_ {}",
                escape(&couch.name.first_name),
                escape(&couch.name.last_name.unwrap_or_default())
            )
        })
        .unwrap_or_default();

    let mut msg = format!(
        "
💪 *Тренировка*: _{}_
📅 *Дата*: _{}_
🧘 *Инструктор*: {}
💁{}
⏱*Продолжительность*: _{}_мин
_{}_                                                                 \n
[Описание]({})
{}
",
        escape(&training.name),
        fmt_dt(&slot.start_at()),
        couch,
        cap,
        training.duration_min,
        status(tr_status, training.is_full()),
        training.description,
        fmt_training_type(training.tp),
    );

    let mut keymap = InlineKeyboardMarkup::default();
    keymap = keymap.append_row(vec![Callback::CouchInfo.button("🧘 Об инструкторе")]);

    if ctx.has_right(Rule::EditTrainingClientsList) {
        keymap = keymap.append_row(vec![Callback::ClientList.button("🗒 Список клиентов")]);
    }

    let mut row = vec![];
    if ctx.has_right(Rule::CancelTraining) {
        if tr_status.can_be_canceled() {
            row.push(Callback::Cancel.button("⛔ Отменить"));
        }
        if tr_status.can_be_uncanceled() {
            row.push(Callback::UnCancel.button("🔓 Вернуть"));
        }
    }
    if ctx.has_right(Rule::SetKeepOpen) {
        if training.keep_open {
            row.push(Callback::KeepOpen(false).button("🔒 Закрыть для записи"));
        } else {
            row.push(Callback::KeepOpen(true).button("🔓 Открыть для записи"));
        }
    }
    keymap = keymap.append_row(row);

    if ctx.has_right(Rule::EditSchedule) {
        keymap = keymap.append_row(vec![
            Callback::Delete(false).button("🗑️ Удалить эту тренировку")
        ]);
        if !training.is_one_time {
            keymap = keymap.append_row(vec![
                Callback::Delete(true).button("🗑️ Удалить все последующие")
            ]);
        }
        keymap = keymap.append_row(vec![
            Callback::ChangeCouchOne.button("🔄 Заменить инструктора")
        ]);
        keymap = keymap.append_row(vec![
            Callback::ChangeCouchAll.button("🔄 Заменить инструктора на все")
        ]);
    }

    if ctx.has_right(Rule::SetFree) {
        if training.tp.is_free() {
            keymap = keymap.append_row(vec![Callback::SetFree(false).button("Сделать платной")]);
        } else {
            keymap = keymap.append_row(vec![Callback::SetFree(true).button("Сделать бесплатной")]);
        }
    }

    if is_client {
        if training.clients.contains(&ctx.me.id) {
            if tr_status.can_sign_out() {
                keymap = keymap.append_row(vec![Callback::SignOut.button("🔓 Отменить запись")]);
            }
        } else if tr_status.can_sign_in() {
            keymap = keymap.append_row(vec![Callback::SignUp.button("🔒 Записаться")]);
        }
    }
    Ok((msg, keymap))
}

#[derive(Serialize, Deserialize)]
enum Callback {
    CouchInfo,
    ChangeCouchOne,
    ChangeCouchAll,
    Delete(bool),
    Cancel,
    ClientList,
    UnCancel,
    SignUp,
    SignOut,
    KeepOpen(bool),
    SetFree(bool),
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
pub struct ConfirmCancelTraining {
    id: DateTime<Local>,
}

impl ConfirmCancelTraining {
    pub fn new(id: DateTime<Local>) -> Self {
        Self { id }
    }

    async fn cancel_training(&mut self, ctx: &mut Context) -> Result<Jmp> {
        ctx.ensure(Rule::CancelTraining)?;
        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let to_notify = ctx
            .ledger
            .cancel_training(&mut ctx.session, &training)
            .await?;
        let msg = format!(
            "Тренировка '{}' в {} *отменена*\\.",
            escape(&training.name),
            fmt_dt(&training.get_slot().start_at())
        );
        for client in to_notify {
            if let Ok(user) = ctx.ledger.get_user(&mut ctx.session, client).await {
                ctx.bot
                    .send_notification_to(ChatId(user.tg_id), &msg)
                    .await?;
            }
        }

        Ok(Jmp::Stay)
    }
}

#[async_trait]
impl View for ConfirmCancelTraining {
    fn name(&self) -> &'static str {
        "ConfirmCancelTraining"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let msg = format!(
            "Вы уверены, что хотите отменить тренировку '{}' в {}?",
            escape(&training.name),
            fmt_dt(&training.get_slot().start_at())
        );
        let mut keymap = InlineKeyboardMarkup::default();
        keymap = keymap.append_row(vec![
            CancelCallback::Cancel.button("✅ Да"),
            CancelCallback::Stay.button("❌ нет"),
        ]);
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            CancelCallback::Cancel => self.cancel_training(ctx).await?,
            CancelCallback::Stay => Jmp::Stay,
        };
        Ok(Jmp::Back)
    }
}

#[derive(Serialize, Deserialize)]
enum CancelCallback {
    Cancel,
    Stay,
}
