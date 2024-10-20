use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::user::fmt_come_from;
use eyre::Result;
use model::{decimal::Decimal, rights::Rule, statistics::marketing::ComeFrom};
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

use crate::FinanceView;

pub struct PayRent;

#[async_trait]
impl View for PayRent {
    fn name(&self) -> &'static str {
        "PayRent"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.edit_origin("Введите сумму оптаты за маркетинг", Default::default())
            .await?;
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, message: &Message) -> Result<Jmp> {
        ctx.delete_msg(message.id).await?;
        let text = if let Some(msg) = message.text() {
            msg
        } else {
            return Ok(Jmp::Stay);
        };

        let amount = match text.parse::<u64>() {
            Ok(amount) => amount,
            Err(_) => {
                ctx.send_msg("Введите число").await?;
                return Ok(Jmp::Stay);
            }
        };

        Ok(Jmp::Next(
            ComeFromType {
                amount: Decimal::int(amount as i64),
            }
            .into(),
        ))
    }
}

struct ComeFromType {
    amount: Decimal,
}

#[async_trait]
impl View for ComeFromType {
    fn name(&self) -> &'static str {
        "ComeFromType"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let msg = "Выберите категорию оплаты".to_string();

        let mut keymap = InlineKeyboardMarkup::default();

        for cf in ComeFrom::iter() {
            keymap = keymap.append_row(cf.btn_row(fmt_come_from(cf)));
        }

        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp> {
        let come_from: ComeFrom = calldata!(data);
        Ok(Jmp::Next(
            Confirm {
                amount: self.amount,
                come_from,
            }
            .into(),
        ))
    }
}

struct Confirm {
    amount: Decimal,
    come_from: ComeFrom,
}

#[async_trait]
impl View for Confirm {
    fn name(&self) -> &'static str {
        "Confirm"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let msg = format!(
            "Подтвердите оплату маркетинга на сумму {}",
            escape(&self.amount.to_string())
        );

        let mut keymap = InlineKeyboardMarkup::default();
        keymap = keymap.append_row(vec![
            Callback::Confirm.button("✅ Подтвердить"),
            Callback::Cancel.button("❌ Отмена"),
        ]);
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::Confirm => {
                ctx.ensure(Rule::MakePayment)?;
                ctx.ledger
                    .treasury
                    .pay_for_marketing(&mut ctx.session, self.amount, self.come_from)
                    .await?;
                ctx.send_msg("Операция выполнена").await?;
                Ok(Jmp::Goto(FinanceView.into()))
            }
            Callback::Cancel => Ok(Jmp::Back),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Confirm,
    Cancel,
}
