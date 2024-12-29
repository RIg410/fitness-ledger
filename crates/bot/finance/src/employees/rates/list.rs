use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::user::render_rate;
use eyre::Result;
use model::{errors::LedgerError, rights::Rule, user::rate::Rate};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

use super::new::CreateRate;

pub struct RatesList {
    id: ObjectId,
    index: usize,
    rate: Option<Rate>,
}

impl RatesList {
    pub fn new(id: ObjectId) -> Self {
        Self {
            id,
            index: 0,
            rate: None,
        }
    }
}

#[async_trait]
impl View for RatesList {
    fn name(&self) -> &'static str {
        "RatesList"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::EditEmployeeRates)?;
        let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
        let employee_info = user
            .employee
            .ok_or_else(|| LedgerError::UserNotEmployee { user_id: self.id })?;
        let mut msg = "Тарифы:".to_string();

        if self.index >= employee_info.rates.len() {
            self.index = employee_info.rates.len().saturating_sub(1);
        }

        for (i, rate) in employee_info.rates.iter().enumerate() {
            let select = if i == self.index {
                self.rate = Some(rate.clone());
                "✅"
            } else {
                "🔸"
            };
            msg.push_str(&format!("\n{} {}", select, render_rate(rate)));
        }

        let mut keymap = InlineKeyboardMarkup::default();

        keymap = keymap.append_row(vec![
            ListCalldata::Prev.button("⬅️"),
            ListCalldata::Next.button("➡️"),
        ]);

        if self.index < employee_info.rates.len() {
            keymap = keymap.append_row(vec![
                ListCalldata::Edit.button("✏️ Редактировать"),
                ListCalldata::Delete.button("❌ Удалить"),
            ]);
        }
        keymap = keymap.append_row(vec![ListCalldata::Create.button("➕ Создать")]);

        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        ctx.ensure(Rule::EditEmployeeRates)?;
        match calldata!(data) {
            ListCalldata::Next => {
                self.index += 1;
                Ok(Jmp::Stay)
            }
            ListCalldata::Prev => {
                self.index = self.index.saturating_sub(1);
                Ok(Jmp::Stay)
            }
            ListCalldata::Edit => {
                let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
                let rate = user.employee.unwrap().rates.get(self.index).unwrap();
                //Ok(RateEdit::new(self.id, rate.clone()).into())
                todo!()
            }
            ListCalldata::Delete => {
                if let Some(rate) = self.rate.clone() {
                    Ok(DeleteRateConfirm::new(self.id, self.index, rate).into())
                } else {
                    Ok(Jmp::Stay)
                }
            }
            ListCalldata::Create => Ok(CreateRate::new(self.id).into()),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum ListCalldata {
    Next,
    Prev,
    Edit,
    Delete,
    Create,
}

pub struct DeleteRateConfirm {
    id: ObjectId,
    idx: usize,
    rate: Rate,
}

impl DeleteRateConfirm {
    pub fn new(id: ObjectId, idx: usize, rate: Rate) -> Self {
        Self { id, idx, rate }
    }
}

#[async_trait]
impl View for DeleteRateConfirm {
    fn name(&self) -> &'static str {
        "DeleteRateConfirm"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::EditEmployeeRates)?;
        let mut keymap = InlineKeyboardMarkup::default();
        keymap = keymap.append_row(vec![
            DeleteRateCalldata::Yes.button("✅ Да"),
            DeleteRateCalldata::No.button("❌ Нет"),
        ]);
        let msg = format!("Удалить тариф?\n{}", render_rate(&self.rate));

        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            DeleteRateCalldata::Yes => {
                ctx.ensure(Rule::EditEmployeeRates)?;
                let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
                let employee_info = user
                    .employee
                    .ok_or_else(|| LedgerError::UserNotEmployee { user_id: self.id })?;
                let same_rate = employee_info
                    .rates
                    .get(self.idx)
                    .map(|r| r == &self.rate)
                    .unwrap_or_default();

                if same_rate {
                    ctx.ledger
                        .users
                        .remove_rate(&mut ctx.session, self.id, self.rate)
                        .await?;
                    Ok(Jmp::Back)
                } else {
                    ctx.send_notification("Тариф был изменен").await?;
                    Ok(Jmp::Back)
                }
            }
            DeleteRateCalldata::No => Ok(Jmp::Back),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum DeleteRateCalldata {
    Yes,
    No,
}
