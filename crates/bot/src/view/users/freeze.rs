use super::View;
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::{eyre, Context as _, Result};
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

pub struct FreezeProfile {
    tg_id: i64,
    go_back: Option<Widget>,
    state: State,
    days: u32,
}

impl FreezeProfile {
    pub fn new(tg_id: i64, go_back: Option<Widget>) -> FreezeProfile {
        FreezeProfile {
            tg_id,
            go_back,
            state: State::SetDays,
            days: 0,
        }
    }
}

#[async_trait]
impl View for FreezeProfile {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        match self.state {
            State::SetDays => {
                let user = ctx
                    .ledger
                    .users
                    .get_by_tg_id(&mut ctx.session, self.tg_id)
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
                    InlineKeyboardButton::callback("✅ Да. Замораживаем", Callback::Yes.to_data()),
                    InlineKeyboardButton::callback("❌ Отмена", Callback::No.to_data()),
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

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        match self.state {
            State::SetDays => {
                let days = message.text().unwrap_or_default();
                match days.parse::<NonZero<u32>>() {
                    Ok(day) => {
                        self.state = State::Confirm;
                        self.days = day.get();
                        self.show(ctx).await?;
                    }
                    Err(_) => {
                        ctx.send_msg("Введите число\\.").await?;
                        self.show(ctx).await?;
                    }
                }
            }
            State::Confirm => {
                ctx.delete_msg(message.id).await?;
            }
        }
        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        let cb = if let Some(cb) = Callback::from_data(data) {
            cb
        } else {
            return Ok(None);
        };
        match cb {
            Callback::Yes => {
                let user = ctx
                    .ledger
                    .users
                    .get_by_tg_id(&mut ctx.session, self.tg_id)
                    .await?
                    .ok_or_else(|| eyre!("User not found!"))?;
                if user.freeze_days < self.days {
                    self.state = State::SetDays;
                    ctx.send_msg("у вас недостаточно дней заморозки").await?;
                    return Ok(None);
                }

                if user.freeze.is_some() {
                    ctx.send_msg("абонемент уже заморожен").await?;
                    let id = ctx.send_msg("\\.").await?;
                    ctx.update_origin_msg_id(id);
                    return Ok(self.go_back.take());
                }
                if !ctx.has_right(Rule::FreezeUsers) && ctx.me.tg_id != self.tg_id {
                    ctx.send_msg("Нет прав").await?;
                    let id = ctx.send_msg("\\.").await?;
                    ctx.update_origin_msg_id(id);
                    return Ok(self.go_back.take());
                }

                ctx.ledger
                    .users
                    .freeze(&mut ctx.session, user.tg_id, self.days)
                    .await
                    .context("freeze")?;
            }
            Callback::No => {}
        }

        let id = ctx.send_msg("\\.").await?;
        ctx.update_origin_msg_id(id);
        return Ok(self.go_back.take());
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
