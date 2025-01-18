use crate::employees::profile::EmployeeProfile;

use super::{fix::FixRateAmount, group::GroupRateMin, personal::PersonalRate};
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::user::render_rate;
use eyre::Result;
use model::{rights::Rule, user::rate::Rate};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

pub struct CreateRate {
    pub employee_id: ObjectId,
}

impl CreateRate {
    pub fn new(employee_id: ObjectId) -> Self {
        Self { employee_id }
    }
}

#[async_trait]
impl View for CreateRate {
    fn name(&self) -> &'static str {
        "CreateRate"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::EditEmployeeRates)?;

        let msg = "Выберите тип тарифа:";
        let mut keymap = InlineKeyboardMarkup::default();
        keymap = keymap.append_row(Callback::Fix.btn_row("Фиксированный"));
        keymap = keymap.append_row(Callback::Group.btn_row("Групповой"));
        keymap = keymap.append_row(Callback::Personal.btn_row("Персональный"));

        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        ctx.ensure(Rule::EditEmployeeRates)?;

        match calldata!(data) {
            Callback::Fix => Ok(Jmp::Next(FixRateAmount::new(None, self.employee_id).into())),
            Callback::Group => Ok(Jmp::Next(GroupRateMin::new(None, self.employee_id).into())),
            Callback::Personal => Ok(Jmp::Next(PersonalRate::new(None, self.employee_id).into())),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Fix,
    Group,
    Personal,
}

pub struct ConfirmCreationRate {
    old_data: Option<Rate>,
    new_data: Rate,
    employee_id: ObjectId,
}

impl ConfirmCreationRate {
    pub fn new(old_data: Option<Rate>, new_data: Rate, employee_id: ObjectId) -> Self {
        Self {
            old_data,
            new_data,
            employee_id,
        }
    }
}

#[async_trait]
impl View for ConfirmCreationRate {
    fn name(&self) -> &'static str {
        "ConfirmCreationRate"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::EditEmployeeRates)?;

        let mut keymap = InlineKeyboardMarkup::default();
        keymap = keymap.append_row(vec![
            ConfirmCallback::Yes.button("✅ Да"),
            ConfirmCallback::No.button("❌ Нет"),
        ]);

        if let Some(old) = &self.old_data {
            let msg = format!(
                "Обновить тариф\nСтарый тариф:\n{}\nНовый тариф:\n{}",
                render_rate(old),
                render_rate(&self.new_data)
            );
            ctx.edit_origin(&msg, keymap).await?;
        } else {
            let msg = format!("Создать тариф?\n{}", render_rate(&self.new_data));
            ctx.edit_origin(&msg, keymap).await?;
        }
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        ctx.ensure(Rule::EditEmployeeRates)?;

        match calldata!(data) {
            ConfirmCallback::Yes => {
                if let Some(old) = &self.old_data {
                    ctx.ledger
                        .users
                        .update_rate(&mut ctx.session, self.employee_id, *old, self.new_data)
                        .await?;
                } else {
                    ctx.ledger
                        .users
                        .add_rate(&mut ctx.session, self.employee_id, self.new_data)
                        .await?;
                };
                ctx.send_notification("Тариф создан").await?;
                Ok(Jmp::Goto(EmployeeProfile::new(self.employee_id).into()))
            }
            ConfirmCallback::No => Ok(Jmp::Stay),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum ConfirmCallback {
    Yes,
    No,
}
