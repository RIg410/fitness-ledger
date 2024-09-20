use std::num::NonZero;

use super::View;
use async_trait::async_trait;
use bot_core::{callback_data::Calldata as _, calldata, context::Context, widget::Jmp};
use eyre::Result;
use model::{decimal::Decimal, rights::Rule};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct EditSubscription {
    id: ObjectId,
    edit_type: EditType,
    state: State,
}

impl EditSubscription {
    pub fn new(id: ObjectId, edit_type: EditType) -> Self {
        Self {
            edit_type,
            state: State::Init,
            id,
        }
    }

    pub async fn edit_price(&self, ctx: &mut Context, value: Decimal) -> Result<Jmp> {
        ctx.ensure(Rule::EditSubscription)?;
        ctx.ledger
            .subscriptions
            .edit_price(&mut ctx.session, self.id, value)
            .await?;
        Ok(Jmp::None)
    }

    pub async fn edit_items(&self, ctx: &mut Context, value: u32) -> Result<Jmp> {
        ctx.ensure(Rule::EditSubscription)?;
        ctx.ledger
            .subscriptions
            .edit_items(&mut ctx.session, self.id, value)
            .await?;
        Ok(Jmp::None)
    }

    pub async fn edit_name(&self, ctx: &mut Context, value: String) -> Result<Jmp> {
        ctx.ensure(Rule::EditSubscription)?;
        ctx.ledger
            .subscriptions
            .edit_name(&mut ctx.session, self.id, value)
            .await?;
        Ok(Jmp::None)
    }
}

#[async_trait]
impl View for EditSubscription {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::EditSubscription)?;
        let keymap = InlineKeyboardMarkup::default();
        match self.edit_type {
            EditType::Name => {
                ctx.send_msg_with_markup("Введите новое название", keymap)
                    .await?;
            }
            EditType::Price => {
                ctx.send_msg_with_markup("Введите новую цену", keymap)
                    .await?;
            }
            EditType::Items => {
                ctx.send_msg_with_markup("Введите новое количество занятий", keymap)
                    .await?;
            }
        }
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, message: &Message) -> Result<Jmp> {
        match self.state {
            State::Init => {
                let text = message.text().unwrap_or_default().to_string();
                let new_value = match self.edit_type {
                    EditType::Items => {
                        if let Err(err) = text.parse::<NonZero<u32>>() {
                            ctx.send_msg(&format!("Неверный формат: {}", err)).await?;
                            return Ok(Jmp::None);
                        }
                        format!("количество занятий на {}", text)
                    }
                    EditType::Price => {
                        if let Err(err) = text.parse::<Decimal>() {
                            ctx.send_msg(&format!("Неверный формат: {}", err)).await?;
                            return Ok(Jmp::None);
                        }
                        format!("цену на {}", text)
                    }
                    EditType::Name => format!("название на {}", text),
                };
                self.state = State::Confirm(text);
                let mut keymap = InlineKeyboardMarkup::default();
                keymap = keymap.append_row(vec![
                    Callback::Yes.button("✅ Да"),
                    Callback::No.button("❌ Нет"),
                ]);
                ctx.send_msg_with_markup(
                    &escape(&format!("Вы уверены, что хотите изменить {}?", new_value)),
                    keymap,
                )
                .await?;
            }
            State::Confirm(_) => {
                ctx.delete_msg(message.id).await?;
            }
        }

        Ok(Jmp::None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::Yes => {
                let value = if let State::Confirm(value) = self.state.clone() {
                    value
                } else {
                    return Ok(Jmp::None);
                };
                match self.edit_type {
                    EditType::Price => self.edit_price(ctx, value.parse()?).await?,
                    EditType::Items => self.edit_items(ctx, value.parse()?).await?,
                    EditType::Name => self.edit_name(ctx, value).await?,
                };
                ctx.send_msg("Изменения сохранены ✅").await?;
                ctx.reset_origin().await?;
                Ok(Jmp::Back)
            }
            Callback::No => {
                ctx.reset_origin().await?;
                Ok(Jmp::Back)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum State {
    Init,
    Confirm(String),
}

#[derive(Clone, Copy)]
pub enum EditType {
    Price,
    Name,
    Items,
}

#[derive(Serialize, Deserialize)]
pub enum Callback {
    Yes,
    No,
}
