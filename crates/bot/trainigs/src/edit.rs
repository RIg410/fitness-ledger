use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata,
    calldata,
    context::Context,
    widget::{Dest, View},
};
use eyre::Result;
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct EditProgram {
    id: ObjectId,
    edit_type: EditType,
    state: State,
}

impl EditProgram {
    pub fn new(id: ObjectId, edit_type: EditType) -> Self {
        Self {
            edit_type,
            state: State::Init,
            id,
        }
    }

    pub async fn edit_capacity(&self, ctx: &mut Context, value: u32) -> Result<Dest> {
        ctx.ensure(Rule::EditTraining)?;
        ctx.ledger
            .edit_program_capacity(&mut ctx.session, self.id, value)
            .await?;
        Ok(Dest::None)
    }

    pub async fn edit_duration(&self, ctx: &mut Context, value: u32) -> Result<Dest> {
        ctx.ensure(Rule::EditTraining)?;
        ctx.ledger
            .edit_program_duration(&mut ctx.session, self.id, value)
            .await?;
        Ok(Dest::None)
    }

    pub async fn edit_name(&self, ctx: &mut Context, value: String) -> Result<Dest> {
        ctx.ensure(Rule::EditTraining)?;
        ctx.ledger
            .edit_program_name(&mut ctx.session, self.id, value)
            .await?;
        Ok(Dest::None)
    }

    pub async fn edit_description(&self, ctx: &mut Context, value: String) -> Result<Dest> {
        ctx.ensure(Rule::EditTraining)?;
        ctx.ledger
            .edit_program_description(&mut ctx.session, self.id, value)
            .await?;
        Ok(Dest::None)
    }
}

#[async_trait]
impl View for EditProgram {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let keymap = InlineKeyboardMarkup::default();
        match self.edit_type {
            EditType::Capacity => {
                ctx.send_msg_with_markup("Введите новую вместимость", keymap)
                    .await?;
            }
            EditType::Duration => {
                ctx.send_msg_with_markup("Введите новую длительность", keymap)
                    .await?;
            }
            EditType::Name => {
                ctx.send_msg_with_markup("Введите новое название", keymap)
                    .await?;
            }
            EditType::Description => {
                ctx.send_msg_with_markup("Введите новое описание", keymap)
                    .await?;
            }
        }
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, message: &Message) -> Result<Dest> {
        match self.state {
            State::Init => {
                let text = message.text().unwrap_or_default().to_string();
                let new_value = match self.edit_type {
                    EditType::Capacity => {
                        if let Err(err) = text.parse::<NonZero<u32>>() {
                            ctx.send_msg(&format!("Неверный формат: {}", err)).await?;
                            return Ok(Dest::None);
                        }
                        format!("вместимость на {}", text)
                    }
                    EditType::Duration => {
                        if let Err(err) = text.parse::<NonZero<u32>>() {
                            ctx.send_msg(&format!("Неверный формат: {}", err)).await?;
                            return Ok(Dest::None);
                        }
                        format!("длительность на {}", text)
                    }
                    EditType::Name => format!("название на {}", text),
                    EditType::Description => format!("описание на {}", text),
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

        Ok(Dest::None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Dest> {
        match calldata!(data) {
            Callback::Yes => {
                let value = if let State::Confirm(value) = self.state.clone() {
                    value
                } else {
                    return Ok(Dest::None);
                };
                match self.edit_type {
                    EditType::Capacity => self.edit_capacity(ctx, value.parse()?).await?,
                    EditType::Duration => self.edit_duration(ctx, value.parse()?).await?,
                    EditType::Name => self.edit_name(ctx, value).await?,
                    EditType::Description => self.edit_description(ctx, value).await?,
                };
                ctx.send_msg("Изменения сохранены ✅").await?;
                ctx.reset_origin().await?;
                Ok(Dest::Back)
            }
            Callback::No => {
                ctx.reset_origin().await?;
                Ok(Dest::Back)
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
    Capacity,
    Duration,
    Name,
    Description,
}

#[derive(Serialize, Deserialize)]
pub enum Callback {
    Yes,
    No,
}
