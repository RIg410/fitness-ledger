use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::Result;
use model::{decimal::Decimal, rights::Rule, user::User};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

#[derive(Default)]
pub struct SelectCouch;

#[async_trait]
impl View for SelectCouch {
    fn name(&self) -> &'static str {
        "SelectCouch"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let msg = "Инструкторы ❤️";
        let mut keymap = InlineKeyboardMarkup::default();
        let instructs = ctx.ledger.users.employees(&mut ctx.session).await?;

        for instruct in instructs {
            keymap = keymap.append_row(vec![render_button(&instruct)]);
        }

        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::SelectCouch(id) => {
                let id: ObjectId = ObjectId::from_bytes(id);
                return Ok(WriteSum::new(id).into());
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    SelectCouch([u8; 12]),
}

fn render_button(user: &User) -> InlineKeyboardButton {
    Callback::SelectCouch(user.id.bytes()).button(format!(
        "{} {}",
        user.name.first_name,
        user.employee.as_ref().map(|c| c.reward).unwrap_or_default()
    ))
}

pub struct WriteSum {
    id: ObjectId,
}

impl WriteSum {
    pub fn new(id: ObjectId) -> WriteSum {
        WriteSum { id }
    }
}

#[async_trait]
impl View for WriteSum {
    fn name(&self) -> &'static str {
        "WriteSum"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let msg = "Введите сумму";
        ctx.edit_origin(msg, InlineKeyboardMarkup::default())
            .await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.delete_msg(msg.id).await?;
        let txt = if let Some(txt) = msg.text() {
            txt
        } else {
            return Ok(Jmp::Stay);
        };
        if let Ok(sum) = txt.parse::<Decimal>() {
            let user = ctx
                .ledger
                .get_user(&mut ctx.session, self.id)
                .await?
                .employee
                .ok_or_else(|| eyre::eyre!("No couch"))?;

            if user.reward < sum {
                ctx.send_msg("Недостаточно средств").await?;
            } else {
                return Ok(ConfirmSum::new(self.id, sum).into());
            }
        } else {
            ctx.send_msg("Неверный формат").await?;
        }

        Ok(Jmp::Stay)
    }
}

pub struct ConfirmSum {
    id: ObjectId,
    sum: Decimal,
}

impl ConfirmSum {
    pub fn new(id: ObjectId, sum: Decimal) -> ConfirmSum {
        ConfirmSum { id, sum }
    }
}

#[async_trait]
impl View for ConfirmSum {
    fn name(&self) -> &'static str {
        "ConfirmSum"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;

        let msg = format!(
            "Выплатить _{}_ пользователю _{}_?",
            escape(&self.sum.to_string()),
            escape(&user.name.first_name)
        );

        let mut keymap = InlineKeyboardMarkup::default();
        keymap = keymap.append_row(vec![ConfirmCallback::Confirm.button("✅ Подтвердить")]);
        keymap = keymap.append_row(vec![ConfirmCallback::Cancel.button("❌ Отмена")]);

        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            ConfirmCallback::Confirm => {
                ctx.ensure(Rule::MakePayment)?;
                ctx.ledger
                    .pay_reward(&mut ctx.session, self.id, self.sum)
                    .await?;
                ctx.send_msg("Операция выполнена").await?;
                Ok(Jmp::Back)
            }
            ConfirmCallback::Cancel => Ok(Jmp::Back),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum ConfirmCallback {
    Confirm,
    Cancel,
}
