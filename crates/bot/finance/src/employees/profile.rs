use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_trainigs::list::TrainingList;
use bot_users::{
    history::HistoryList, rewards::RewardsList, rights::UserRightsView, set_birthday::SetBirthday,
    set_fio::SetFio, set_phone::SetPhone,
};
use bot_viewer::user::render_profile_msg;
use eyre::Error;
use model::{rights::Rule, user::rate::EmployeeRole};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};

use super::{delete::DeleteEmployeeConfirm, rates::list::RatesList, reward::PayReward};

pub struct EmployeeProfile {
    id: ObjectId,
}

impl EmployeeProfile {
    pub fn new(id: ObjectId) -> EmployeeProfile {
        EmployeeProfile { id }
    }

    async fn block_user(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::BlockUser)?;
        let user = ctx
            .ledger
            .users
            .get(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("User not found"))?;
        ctx.ledger
            .block_user(&mut ctx.session, self.id, !user.is_active)
            .await?;
        ctx.reload_user().await?;
        Ok(Jmp::Stay)
    }

    async fn edit_rights(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::EditUserRights)?;
        Ok(UserRightsView::new(self.id).into())
    }

    async fn set_birthday(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        if ctx.has_right(Rule::EditUserInfo) || ctx.me.id == self.id {
            Ok(SetBirthday::new(self.id).into())
        } else {
            Ok(Jmp::Stay)
        }
    }

    async fn training_list(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
        if user.employee.is_some() {
            Ok(TrainingList::couches(user.id).into())
        } else {
            Ok(TrainingList::users(user.id).into())
        }
    }

    async fn history_list(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
        Ok(HistoryList::new(user.id).into())
    }

    async fn rewards_list(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
        if user.employee.is_some() && (ctx.is_me(user.id) || ctx.has_right(Rule::ViewRewards)) {
            Ok(RewardsList::new(user.id).into())
        } else {
            Ok(Jmp::Stay)
        }
    }

    async fn rates_list(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::EditEmployeeRates)?;
        Ok(Jmp::Next(RatesList::new(self.id).into()))
    }

    async fn pay_reward(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::MakePayment)?;
        Ok(Jmp::Next(PayReward::new(self.id).into()))
    }

    async fn delete_employee(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::EditEmployee)?;
        Ok(Jmp::Next(DeleteEmployeeConfirm::new(self.id).into()))
    }

    async fn set_fio(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        if ctx.has_right(Rule::EditUserInfo) {
            Ok(SetFio::new(self.id).into())
        } else {
            Ok(Jmp::Stay)
        }
    }

    async fn set_phone(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        if ctx.has_right(Rule::EditUserInfo) {
            Ok(SetPhone::new(self.id).into())
        } else {
            Ok(Jmp::Stay)
        }
    }
}

#[async_trait]
impl View for EmployeeProfile {
    fn name(&self) -> &'static str {
        "EmployeeProfile"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let (msg, keymap) = render_user_profile(ctx, self.id).await?;
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
            Callback::SetBirthday => self.set_birthday(ctx).await,
            Callback::EditPhone => self.set_phone(ctx).await,
            Callback::TrainingList => self.training_list(ctx).await,
            Callback::HistoryList => self.history_list(ctx).await,
            Callback::RewardsList => self.rewards_list(ctx).await,
            Callback::DeleteEmployee => self.delete_employee(ctx).await,
            Callback::PayReward => self.pay_reward(ctx).await,
            Callback::Rates => self.rates_list(ctx).await,
        }
    }
}

async fn render_user_profile(
    ctx: &mut Context,
    id: ObjectId,
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let (msg, user, extension) = render_profile_msg(ctx, id).await?;

    let mut keymap = InlineKeyboardMarkup::default();

    if extension.birthday.is_none() || ctx.has_right(Rule::EditUserInfo) {
        keymap = keymap.append_row(Callback::SetBirthday.btn_row("Установить дату рождения 🎂"));
    }

    if let Some(employee) = user.employee.as_ref() {
        if employee.role == EmployeeRole::Couch {
            keymap = keymap.append_row(Callback::TrainingList.btn_row("Тренировки 📝"));
        }
    }

    if ctx.has_right(Rule::BlockUser) && ctx.me.tg_id != user.tg_id {
        keymap = keymap.append_row(Callback::BlockUnblock.btn_row(if user.is_active {
            "Заблокировать ❌"
        } else {
            "Разблокировать ✅"
        }));
    }
    if ctx.has_right(Rule::EditEmployeeRates) {
        keymap = keymap.append_row(Callback::Rates.btn_row("Тарифы 💰"));
    }
    if ctx.has_right(Rule::EditUserInfo) {
        keymap = keymap.append_row(Callback::EditFio.btn_row("✍️ Редактировать ФИО"));
        keymap = keymap.append_row(Callback::EditPhone.btn_row("✍️ Редактировать телефон"));
    }

    if ctx.has_right(Rule::EditUserRights) {
        keymap = keymap.append_row(Callback::EditRights.btn_row("Права 🔒"));
    }
    keymap = keymap.append_row(Callback::HistoryList.btn_row("История 📝"));
    if user.employee.is_some() && (ctx.is_me(id) || ctx.has_right(Rule::ViewRewards)) {
        keymap = keymap.append_row(Callback::RewardsList.btn_row("Вознаграждения 📝"));
    }
    if ctx.has_right(Rule::MakePayment) {
        keymap = keymap.append_row(Callback::PayReward.btn_row("Выплатить вознаграждение 💰"));
    }

    if ctx.has_right(Rule::EditEmployee) {
        keymap = keymap.append_row(Callback::DeleteEmployee.btn_row("Удалить сотрудника ❌"));
    }

    Ok((msg, keymap))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Callback {
    BlockUnblock,
    EditFio,
    EditPhone,
    SetBirthday,
    EditRights,
    TrainingList,
    HistoryList,
    RewardsList,
    DeleteEmployee,
    PayReward,
    Rates,
}
