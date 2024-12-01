use crate::SubscriptionView;

use super::{confirm::ConfirmSell, View};
use async_trait::async_trait;
use bot_core::{callback_data::Calldata as _, calldata, context::Context, widget::Jmp};
use bot_viewer::{fmt_phone, user::fmt_come_from};
use eyre::Result;
use model::{
    request::Request, rights::Rule, statistics::marketing::ComeFrom, user::sanitize_phone,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub const LIMIT: u64 = 7;

pub struct SellView {
    sell: ObjectId,
    state: SellViewState,
}

impl SellView {
    pub fn new(sell: ObjectId) -> SellView {
        SellView {
            sell,
            state: SellViewState::SelectUser,
        }
    }
}

#[async_trait]
impl View for SellView {
    fn name(&self) -> &'static str {
        "SellView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let mut text = String::new();
        let mut keymap = InlineKeyboardMarkup::default();

        match &self.state {
            SellViewState::SelectUser => {
                text = "Введите номер телефона пользователя".to_string();
            }
            SellViewState::FindByPhone(phone) => {
                if ctx
                    .ledger
                    .users
                    .get_by_phone(&mut ctx.session, &phone)
                    .await?
                    .is_none()
                {
                    text = format!(
                        "Пользователь с номером *{}* не найден\\. Создать нового пользователя?",
                        fmt_phone(Some(&phone))
                    );
                    keymap = keymap.append_row(SellViewCallback::CreateNewUser.btn_row("Создать"));
                }
            }
        }

        ctx.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, msg: &Message) -> Result<Jmp> {
        ctx.delete_msg(msg.id).await?;
        let query = msg.text().unwrap_or_default();

        if query.starts_with("8") {
            let query = "7".to_string() + &query[1..];
            self.state = SellViewState::FindByPhone(sanitize_phone(&query));
        } else if query.starts_with("+7") {
            self.state = SellViewState::FindByPhone(sanitize_phone(&query));
        } else {
            ctx.send_msg("Номер телефона должен начинаться с 8 или \\+7")
                .await?;
            return Ok(Jmp::Stay);
        }

        if let SellViewState::FindByPhone(phone) = &self.state {
            if let Some(user) = ctx
                .ledger
                .users
                .get_by_phone(&mut ctx.session, phone)
                .await?
            {
                return Ok(Jmp::Next(ConfirmSell::new(user.id, self.sell).into()));
            }
        }

        Ok(Jmp::Stay)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        ctx.ensure(Rule::SellSubscription)?;
        match calldata!(data) {
            SellViewCallback::CreateNewUser => {
                if let SellViewState::FindByPhone(phone) = &self.state {
                    return Ok(Jmp::Next(SetName::new(self.sell, phone.clone()).into()));
                }
            }
        }
        Ok(Jmp::Stay)
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum SellViewCallback {
    CreateNewUser,
}

enum SellViewState {
    SelectUser,
    FindByPhone(String),
}

struct SetName {
    sell: ObjectId,
    phone: String,
}

impl SetName {
    pub fn new(sell: ObjectId, phone: String) -> SetName {
        SetName { sell, phone }
    }
}

#[async_trait]
impl View for SetName {
    fn name(&self) -> &'static str {
        "CreateUser"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.edit_origin(
            "Введите имя пользователя\\.",
            InlineKeyboardMarkup::default(),
        )
        .await?;
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, msg: &Message) -> Result<Jmp> {
        ctx.delete_msg(msg.id).await?;
        let name = msg.text().unwrap_or_default();
        if name.is_empty() {
            ctx.send_msg("Имя не может быть пустым").await?;
            return Ok(Jmp::Stay);
        }

        let parts: Vec<_> = name.split(' ').collect();
        let first_name = parts.get(0).unwrap_or(&"").to_string();
        let last_name = parts.get(1).map(|s| s.to_string());

        Ok(Jmp::Next(
            SelectComeFrom::new(self.sell, self.phone.clone(), first_name, last_name).into(),
        ))
    }
}

pub struct SelectComeFrom {
    sell: ObjectId,
    phone: String,
    first_name: String,
    last_name: Option<String>,
}

impl SelectComeFrom {
    pub fn new(
        sell: ObjectId,
        phone: String,
        first_name: String,
        last_name: Option<String>,
    ) -> SelectComeFrom {
        SelectComeFrom {
            sell,
            phone,
            first_name,
            last_name,
        }
    }
}

#[async_trait]
impl View for SelectComeFrom {
    fn name(&self) -> &'static str {
        "SelectFrom"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let mut markup = InlineKeyboardMarkup::default();
        for come_from in ComeFrom::iter() {
            markup = markup.append_row(come_from.btn_row(fmt_come_from(come_from)));
        }
        ctx.edit_origin("Выберите откуда пришел пользователь:", markup)
            .await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        ctx.ensure(Rule::SellSubscription)?;
        let come_from = calldata!(data);
        Ok(Jmp::Next(
            CreateUserAndSell::new(
                self.sell,
                self.phone.clone(),
                self.first_name.clone(),
                self.last_name.clone(),
                come_from,
            )
            .into(),
        ))
    }
}

pub struct CreateUserAndSell {
    sell: ObjectId,
    phone: String,
    first_name: String,
    last_name: Option<String>,
    come_from: ComeFrom,
}

impl CreateUserAndSell {
    pub fn new(
        sell: ObjectId,
        phone: String,
        first_name: String,
        last_name: Option<String>,
        come_from: ComeFrom,
    ) -> CreateUserAndSell {
        CreateUserAndSell {
            sell,
            phone,
            first_name,
            last_name,
            come_from,
        }
    }
}

#[async_trait]
impl View for CreateUserAndSell {
    fn name(&self) -> &'static str {
        "CreateUserAndSell"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let sub = ctx
            .ledger
            .subscriptions
            .get(&mut ctx.session, self.sell)
            .await?
            .ok_or_else(|| eyre::eyre!("Subscription {} not found", self.sell))?;

        let text = format!(
            "
 📌  Продажа
Тариф: *{}*\nКоличество занятий:*{}*\nЦена:*{}*\n
Пользователь:
    Имя:*{}*
    Фамилия:*{}*
    Номер:*{}*
    Источник: *{}*\n\n
    Все верно? 
    ",
            escape(&sub.name),
            sub.items,
            sub.price.to_string().replace(".", ","),
            escape(&self.first_name),
            escape(&self.last_name.clone().unwrap_or_else(|| "-".to_string())),
            fmt_phone(Some(&self.phone)),
            fmt_come_from(self.come_from)
        );

        let mut keymap = InlineKeyboardMarkup::default();
        keymap = keymap.append_row(vec![
            Callback::Sell.button("✅ Да"),
            Callback::Cancel.button("❌ Отмена"),
        ]);
        ctx.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::Sell => {
                ctx.ensure(Rule::SellSubscription)?;
                let result = ctx
                    .ledger
                    .presell_subscription(
                        &mut ctx.session,
                        self.sell,
                        self.phone.clone(),
                        self.first_name.clone(),
                        self.last_name.clone(),
                        self.come_from,
                    )
                    .await;

                let request = ctx
                    .ledger
                    .requests
                    .get_by_phone(&mut ctx.session, &self.phone)
                    .await?;
                if request.is_none() {
                    ctx.ledger
                        .requests
                        .create(
                            &mut ctx.session,
                            Request::new(
                                self.phone.clone(),
                                "Продано 🤑".to_string(),
                                self.come_from,
                                Some(self.first_name.clone()),
                                self.last_name.clone(),
                                None,
                            ),
                        )
                        .await?;
                }

                if let Err(err) = result {
                    Err(err.into())
                } else {
                    ctx.send_msg("🤑 Продано").await?;
                    ctx.reset_origin().await?;
                    Ok(Jmp::Goto(SubscriptionView.into()))
                }
            }
            Callback::Cancel => Ok(Jmp::Back),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Sell,
    Cancel,
}
