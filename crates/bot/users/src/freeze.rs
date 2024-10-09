use super::View;
use async_trait::async_trait;
use bot_core::{callback_data::Calldata as _, calldata, context::Context, widget::Jmp};
use eyre::{eyre, Context as _, Result};
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use teloxide::types::{InlineKeyboardMarkup, Message};

pub struct FreezeProfile {
    id: ObjectId,
    state: State,
    days: u32,
}

impl FreezeProfile {
    pub fn new(id: ObjectId) -> FreezeProfile {
        FreezeProfile {
            id,
            state: State::SetDays,
            days: 0,
        }
    }
}

#[async_trait]
impl View for FreezeProfile {
    fn name(&self) -> &'static str {
        "FreezeProfile"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        match self.state {
            State::SetDays => {
                let user = ctx
                    .ledger
                    .users
                    .get(&mut ctx.session, self.id)
                    .await?
                    .ok_or_else(|| eyre!("User not found!"))?;
                ctx.send_msg_with_markup(
                    &format!(
                        "Осталось дней заморозок:_{}_\nНа сколько дней заморозить абонемент?",
                        user.freeze_days
                    ),
                    InlineKeyboardMarkup::default(),
                )
                .await?;
            }
            State::Confirm => {
                let keymap = vec![vec![
                    Callback::Yes.button("✅ Да. Замораживаем"),
                    Callback::No.button("❌ Отмена"),
                ]];
                ctx.send_msg_with_markup(
                    &format!(
                        "Замораживаем Ваш абонемент\\. Количество дней:_{}_\nВсе верно?",
                        self.days
                    ),
                    InlineKeyboardMarkup::new(keymap),
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, message: &Message) -> Result<Jmp> {
        match self.state {
            State::SetDays => {
                let days = message.text().unwrap_or_default();
                match days.parse::<NonZero<u32>>() {
                    Ok(day) => {
                        self.state = State::Confirm;
                        self.days = day.get();
                    }
                    Err(_) => {
                        ctx.send_msg("Введите число\\.").await?;
                    }
                }
            }
            State::Confirm => {
                ctx.delete_msg(message.id).await?;
            }
        }
        Ok(Jmp::Stay)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        let cb = calldata!(data);

        match cb {
            Callback::Yes => {
                let user = ctx
                    .ledger
                    .users
                    .get(&mut ctx.session, self.id)
                    .await?
                    .ok_or_else(|| eyre!("User not found!"))?;
                if user.freeze_days < self.days {
                    self.state = State::SetDays;
                    ctx.send_msg("у вас недостаточно дней заморозки").await?;
                    return Ok(Jmp::Stay);
                }

                if user.freeze.is_some() {
                    ctx.send_msg("абонемент уже заморожен").await?;
                    return Ok(Jmp::Back);
                }
                if !ctx.has_right(Rule::FreezeUsers) && ctx.me.id != self.id {
                    ctx.send_msg("Нет прав").await?;
                    return Ok(Jmp::Back);
                }

                ctx.ledger
                    .users
                    .freeze(&mut ctx.session, user.id, self.days)
                    .await
                    .context("freeze")?;
            }
            Callback::No => {}
        }
        return Ok(Jmp::Back);
    }
}

#[derive(Clone, Copy)]
enum State {
    SetDays,
    Confirm,
}

#[derive(Serialize, Deserialize)]
pub enum Callback {
    Yes,
    No,
}
