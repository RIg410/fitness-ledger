use super::View;
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use client_list::ClientList;
use eyre::{bail, Result};
use model::{
    rights::Rule,
    training::{Training, TrainingStatus},
};
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::Requester as _,
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

mod add_client;
mod client;
mod client_list;

pub struct TrainingView {
    id: DateTime<Local>,
    go_back: Option<Widget>,
}

impl TrainingView {
    pub fn new(id: DateTime<Local>, go_back: Option<Widget>) -> Self {
        Self { id, go_back }
    }

    async fn go_back(&mut self, _: &mut Context) -> Result<Option<Widget>> {
        Ok(self.go_back.take())
    }

    async fn couch_info(&mut self, ctx: &mut Context) -> Result<Option<Widget>> {
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
            let id = ctx.send_msg("\\.").await?;
            ctx.update_origin_msg_id(id);
            self.show(ctx).await?;
        }
        Ok(None)
    }

    async fn cancel_training(&mut self, ctx: &mut Context) -> Result<Option<Widget>> {
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

    async fn delete_training(&mut self, ctx: &mut Context, all: bool) -> Result<Option<Widget>> {
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

    async fn restore_training(&mut self, ctx: &mut Context) -> Result<Option<Widget>> {
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

    async fn sign_up(&mut self, ctx: &mut Context) -> Result<Option<Widget>> {
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
            .sign_up(&mut ctx.session, &training, ctx.me.id, false)
            .await?;
        self.show(ctx).await?;
        Ok(None)
    }

    async fn sign_out(&mut self, ctx: &mut Context) -> Result<Option<Widget>> {
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
            .sign_out(&mut ctx.session, &training, ctx.me.id, false)
            .await?;
        self.show(ctx).await?;
        Ok(None)
    }

    async fn client_list(&mut self, ctx: &mut Context) -> Result<Option<Widget>> {
        if !ctx.is_couch() {
            bail!("Only couch can see client list");
        }
        Ok(Some(ClientList::new(self.id, Some(self.take())).boxed()))
    }

    async fn change_couch(&mut self, ctx: &mut Context) -> Result<Option<Widget>> {
        ctx.ensure(Rule::EditSchedule)?;
        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        // let back = TrainingView::new(self.id, self.go_back.take());
        // Ok(Some(Box::new(add_client::AddClientView::new(
        //     self.id,
        //     Some(Box::new(back)),
        // )))
        todo!()
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
        let (msg, keymap) = render(ctx, &training, self.go_back.is_some()).await?;
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
            Callback::Back => self.go_back(ctx).await,
            Callback::CouchInfo => self.couch_info(ctx).await,
            Callback::Cancel => self.cancel_training(ctx).await,
            Callback::Delete(all) => self.delete_training(ctx, all).await,
            Callback::UnCancel => self.restore_training(ctx).await,
            Callback::SignUp => self.sign_up(ctx).await,
            Callback::SignOut => self.sign_out(ctx).await,
            Callback::ClientList => self.client_list(ctx).await,
            Callback::ChangeCouch => self.change_couch(ctx).await,
        }
    }

    fn take(&mut self) -> Widget {
        TrainingView {
            id: self.id,
            go_back: self.go_back.take(),
        }
        .boxed()
    }
}

async fn render(
    ctx: &mut Context,
    training: &Training,
    has_back: bool,
) -> Result<(String, InlineKeyboardMarkup)> {
    let is_client = ctx.me.couch.is_none();
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

    let msg = format!(
        "
💪 *Тренировка*: _{}_
📅 *Дата*: _{}_
🧘 *Инструктор*: {}
💁{}
⏱*продолжительность*: _{}_мин
_{}_                                                                 \n
[Описание]({})
",
        escape(&training.name),
        slot.start_at().format("%d\\.%m\\.%Y %H:%M"),
        couch,
        cap,
        training.duration_min,
        status(tr_status, training.is_full()),
        training.description,
    );
    let mut keymap = InlineKeyboardMarkup::default();
    keymap = keymap.append_row(vec![Callback::CouchInfo.button("🧘 Об инструкторе")]);

    if !is_client {
        keymap = keymap.append_row(vec![Callback::ClientList.button("🗒 Список клиентов")]);
    }

    if ctx.has_right(Rule::CancelTraining) {
        if tr_status.can_be_canceled() {
            keymap = keymap.append_row(vec![Callback::Cancel.button("⛔ Отменить")]);
        }
        if tr_status.can_be_uncanceled() {
            keymap = keymap.append_row(vec![Callback::UnCancel.button("🔓 Вернуть")]);
        }
    }

    if ctx.has_right(Rule::EditSchedule) {
        let mut keys = vec![];
        keys.push(Callback::Delete(false).button("🗑️ Удалить эту тренировку"));
        if !training.is_one_time {
            keys.push(Callback::Delete(true).button("🗑️ Удалить все последующие"));
        }
        keymap = keymap.append_row(keys);
        keymap = keymap.append_row(vec![Callback::ChangeCouch.button("🔄 Заменить инструктора")]);
    }

    if is_client {
        if training.clients.contains(&ctx.me.id) {
            if tr_status.can_sign_out() {
                keymap = keymap.append_row(vec![Callback::SignOut.button("🔓 Отменить запись")]);
            }
        } else {
            if tr_status.can_sign_in() {
                keymap = keymap.append_row(vec![Callback::SignUp.button("🔒 Записаться")]);
            }
        }
    }
    if has_back {
        keymap = keymap.append_row(vec![Callback::Back.button("🔙 Назад")]);
    }
    Ok((msg, keymap))
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Back,
    CouchInfo,
    ChangeCouch,
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
