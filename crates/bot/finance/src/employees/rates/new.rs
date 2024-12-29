use super::{fix::FixRate, group::GroupRate, personal::PersonalRate};
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::Result;
use model::rights::Rule;
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
            Callback::Fix => Ok(Jmp::Next(FixRate::new(None, self.employee_id).into())),
            Callback::Group => Ok(Jmp::Next(GroupRate::new(None, self.employee_id).into())),
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
