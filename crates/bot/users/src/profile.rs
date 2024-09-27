use crate::history::HistoryList;

use super::{
    freeze::FreezeProfile, rights::UserRightsView, set_birthday::SetBirthday, set_fio::SetFio,
    set_phone::SetPhone,
};
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_trainigs::list::TrainingList;
use bot_viewer::user::render_profile_msg;
use eyre::Error;
use model::{rights::Rule, user::UserIdent};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};

pub struct UserProfile {
    tg_id: i64,
}

impl UserProfile {
    pub fn new(tg_id: i64) -> UserProfile {
        UserProfile { tg_id }
    }

    async fn block_user(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::BlockUser)?;
        let user = ctx
            .ledger
            .users
            .get_by_tg_id(&mut ctx.session, self.tg_id)
            .await?
            .ok_or_else(|| eyre::eyre!("User not found"))?;
        ctx.ledger
            .block_user(&mut ctx.session, self.tg_id, !user.is_active)
            .await?;
        ctx.reload_user().await?;
        Ok(Jmp::Stay)
    }

    async fn change_balance(&mut self, ctx: &mut Context, amount: i32) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::ChangeBalance)?;
        let user = ctx
            .ledger
            .users
            .get_by_tg_id(&mut ctx.session, self.tg_id)
            .await?
            .ok_or_else(|| eyre::eyre!("User not found"))?;

        if amount < 0 && user.balance < amount.unsigned_abs() {
            return Err(eyre::eyre!("Not enough balance"));
        }

        ctx.ledger
            .users
            .change_balance(&mut ctx.session, user.tg_id, amount)
            .await?;
        ctx.reload_user().await?;
        Ok(Jmp::Stay)
    }

    async fn change_reserved_balance(
        &mut self,
        ctx: &mut Context,
        amount: i32,
    ) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::ChangeBalance)?;
        let user = ctx.ledger.get_user(&mut ctx.session, self.tg_id).await?;

        if amount < 0 && user.reserved_balance < amount.unsigned_abs() {
            return Err(eyre::eyre!("Not enough reserved balance"));
        }

        ctx.ledger
            .users
            .change_reserved_balance(&mut ctx.session, user.tg_id, amount)
            .await?;
        ctx.reload_user().await?;
        Ok(Jmp::Stay)
    }

    async fn freeze_user(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        if !ctx.has_right(Rule::FreezeUsers) && ctx.me.tg_id != self.tg_id {
            return Err(eyre::eyre!("User has no rights to perform this action"));
        }
        Ok(FreezeProfile::new(self.tg_id).into())
    }

    async fn edit_rights(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::EditUserRights)?;
        Ok(UserRightsView::new(self.tg_id).into())
    }

    async fn set_birthday(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        if ctx.has_right(Rule::EditUserInfo) || ctx.me.tg_id == self.tg_id {
            Ok(SetBirthday::new(self.tg_id).into())
        } else {
            Ok(Jmp::Stay)
        }
    }

    async fn training_list(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        let user = ctx.ledger.get_user(&mut ctx.session, self.tg_id).await?;
        if user.is_couch() {
            Ok(TrainingList::couches(user.id).into())
        } else {
            Ok(TrainingList::users(user.id).into())
        }
    }

    async fn history_list(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        let user = ctx.ledger.get_user(&mut ctx.session, self.tg_id).await?;
        Ok(HistoryList::new(user.id).into())
    }

    async fn set_fio(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        if ctx.has_right(Rule::EditUserInfo) {
            Ok(SetFio::new(self.tg_id).into())
        } else {
            Ok(Jmp::Stay)
        }
    }

    async fn set_phone(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        if ctx.has_right(Rule::EditUserInfo) {
            Ok(SetPhone::new(self.tg_id).into())
        } else {
            Ok(Jmp::Stay)
        }
    }
}

#[async_trait]
impl View for UserProfile {
    fn name(&self) -> &'static str {
        "UserProfile"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let (msg, keymap) = render_user_profile(ctx, self.tg_id).await?;
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.delete_msg(message.id).await?;
        Ok(Jmp::Stay)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            Callback::BlockUnblock => self.block_user(ctx).await,
            Callback::EditFio => self.set_fio(ctx).await,
            Callback::EditRights => self.edit_rights(ctx).await,
            Callback::Freeze => self.freeze_user(ctx).await,
            Callback::ChangeBalance(amount) => self.change_balance(ctx, amount).await,
            Callback::ChangeReservedBalance(amount) => {
                self.change_reserved_balance(ctx, amount).await
            }
            Callback::SetBirthday => self.set_birthday(ctx).await,
            Callback::EditPhone => self.set_phone(ctx).await,
            Callback::TrainingList => self.training_list(ctx).await,
            Callback::HistoryList => self.history_list(ctx).await,
        }
    }
}

async fn render_user_profile<ID: Into<UserIdent> + Copy>(
    ctx: &mut Context,
    id: ID,
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let (msg, user) = render_profile_msg(ctx, id).await?;

    let mut keymap = InlineKeyboardMarkup::default();
    if (ctx.has_right(Rule::FreezeUsers)
        || ctx.me.tg_id == user.tg_id || !user.subscriptions.is_empty()) && user.freeze.is_none() && user.freeze_days != 0 {
        keymap = keymap.append_row(Callback::Freeze.btn_row("Заморозить ❄"));
    }

    if ctx.has_right(Rule::ChangeBalance) {
        keymap = keymap.append_row(vec![
            Callback::ChangeBalance(-1).button("Списать баланс 💸"),
            Callback::ChangeBalance(1).button("Пополнить баланс 💰"),
        ]);
        keymap = keymap.append_row(vec![
            Callback::ChangeReservedBalance(-1).button("Списать зарезервированный баланс 💸"),
            Callback::ChangeReservedBalance(1).button("Пополнить зарезервированный баланс 💰"),
        ]);
    }

    if user.is_couch() {
        keymap = keymap.append_row(Callback::TrainingList.btn_row("Тренировки 📝"));
    } else {
        keymap = keymap.append_row(Callback::TrainingList.btn_row("Записи 📝"));
    }

    if ctx.has_right(Rule::BlockUser) && ctx.me.tg_id != user.tg_id {
        keymap = keymap.append_row(Callback::BlockUnblock.btn_row(if user.is_active {
            "❌ Заблокировать"
        } else {
            "✅ Разблокировать"
        }));
    }
    if ctx.has_right(Rule::EditUserInfo) || (ctx.me.id == user.id && user.birthday.is_none()) {
        keymap = keymap.append_row(Callback::SetBirthday.btn_row("Установить дату рождения"));
    }

    if ctx.has_right(Rule::EditUserInfo) {
        keymap = keymap.append_row(Callback::EditFio.btn_row("✍️ Редактировать ФИО"));
        keymap = keymap.append_row(Callback::EditPhone.btn_row("✍️ Редактировать телефон"));
    }
    if ctx.has_right(Rule::EditUserRights) {
        keymap = keymap.append_row(Callback::EditRights.btn_row("🔒 Права"));
    }
    keymap = keymap.append_row(Callback::HistoryList.btn_row("История 📝"));
    Ok((msg, keymap))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Callback {
    BlockUnblock,
    EditFio,
    EditPhone,
    SetBirthday,
    EditRights,
    Freeze,
    TrainingList,
    HistoryList,
    ChangeBalance(i32),
    ChangeReservedBalance(i32),
}
