use crate::Marketing;
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::{fmt_phone, user::fmt_come_from};
use model::{rights::Rule, statistics::marketing::ComeFrom, user::sanitize_phone};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct SetPhone;

#[async_trait]
impl View for SetPhone {
    fn name(&self) -> &'static str {
        "SetPhone"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        let text = "Введите номер телефона";
        ctx.bot.edit_origin(text, Default::default()).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.bot.delete_msg(msg.id).await?;
        let mut phone = msg.text().unwrap_or_default().to_string();
        if phone.is_empty() {
            return Ok(Jmp::Stay);
        }

        if phone.starts_with("8") {
            phone = "7".to_string() + &phone[1..];
        }

        phone = sanitize_phone(&phone);

        Ok(Jmp::Next(SetComeFrom { phone }.into()))
    }
}

pub struct SetComeFrom {
    phone: String,
}

#[async_trait]
impl View for SetComeFrom {
    fn name(&self) -> &'static str {
        "SetComeFrom"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        let text = "Откуда пришел клиент?";

        let mut markup = InlineKeyboardMarkup::default();
        for come_from in ComeFrom::iter() {
            markup = markup.append_row(come_from.btn_row(fmt_come_from(come_from)));
        }

        ctx.bot.edit_origin(text, markup).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        let come_from: ComeFrom = calldata!(data);
        Ok(Jmp::Next(
            SetDescription {
                phone: self.phone.clone(),
                come_from,
            }
            .into(),
        ))
    }
}

pub struct SetDescription {
    phone: String,
    come_from: ComeFrom,
}

#[async_trait]
impl View for SetDescription {
    fn name(&self) -> &'static str {
        "SetDescription"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        let text = "Можно оставить комментарий или \\- если нечего добавить";
        ctx.bot.edit_origin(text, Default::default()).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.bot.delete_msg(msg.id).await?;
        let comment = msg.text().unwrap_or_default().to_string();
        Ok(Jmp::Next(
            SetName {
                phone: self.phone.clone(),
                come_from: self.come_from,
                comment: comment.clone(),
            }
            .into(),
        ))
    }
}

pub struct SetName {
    phone: String,
    come_from: ComeFrom,
    comment: String,
}

#[async_trait]
impl View for SetName {
    fn name(&self) -> &'static str {
        "SetName"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        let text = "Введите имя и фамилию";
        ctx.bot.edit_origin(text, Default::default()).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.bot.delete_msg(msg.id).await?;
        let name = msg.text().unwrap_or_default();
        let parts: Vec<_> = name.split(' ').collect();
        let first_name = parts.get(0).map(|s| s.to_string());
        let last_name = parts.get(1).map(|s| s.to_string());
        Ok(Jmp::Next(
            Comfirm {
                phone: self.phone.clone(),
                come_from: self.come_from,
                comment: self.comment.clone(),
                first_name,
                last_name,
            }
            .into(),
        ))
    }
}

pub struct Comfirm {
    phone: String,
    come_from: ComeFrom,
    comment: String,
    first_name: Option<String>,
    last_name: Option<String>,
}

#[async_trait]
impl View for Comfirm {
    fn name(&self) -> &'static str {
        "Comfirm"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        let text = format!(
            "Все верно?:\n\
            Телефон: *{}*\n\
            Откуда пришел: *{}*\n\
            Комментарий: *{}*\n",
            fmt_phone(&self.phone),
            fmt_come_from(self.come_from),
            escape(&self.comment)
        );
        let mut markup = InlineKeyboardMarkup::default();
        markup = markup.append_row(vec![
            CalldataYesNo::Yes.button("✅Да"),
            CalldataYesNo::No.button("❌Нет"),
        ]);
        ctx.bot.edit_origin(&text, markup).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            CalldataYesNo::Yes => {
                ctx.ensure(Rule::CreateRequest)?;
                ctx.ledger
                    .create_request(
                        &mut ctx.session,
                        self.phone.clone(),
                        self.come_from,
                        self.comment.clone(),
                        self.first_name.clone(),
                        self.last_name.clone(),
                    )
                    .await?;
                ctx.send_msg("Заявка создана").await?;

                if ctx.has_right(Rule::SellSubscription) {
                    Ok(Jmp::Next(
                        SellSubscription {
                            phone: self.phone.clone(),
                            come_from: self.come_from,
                            first_name: self.first_name.clone(),
                            last_name: self.last_name.clone(),
                        }
                        .into(),
                    ))
                } else {
                    Ok(Jmp::Goto(Marketing {}.into()))
                }
            }
            CalldataYesNo::No => Ok(Jmp::Goto(Marketing {}.into())),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum CalldataYesNo {
    Yes,
    No,
}

pub struct SellSubscription {
    pub phone: String,
    pub come_from: ComeFrom,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[async_trait]
impl View for SellSubscription {
    fn name(&self) -> &'static str {
        "SellSubscription"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        let text = "Продать абонемент?";
        let mut markup = InlineKeyboardMarkup::default();
        markup = markup.append_row(vec![
            CalldataYesNo::Yes.button("✅Да"),
            CalldataYesNo::No.button("❌Нет"),
        ]);
        ctx.bot.edit_origin(&text, markup).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            CalldataYesNo::Yes => Ok(Jmp::Next(
                SelectSubscriptionsView {
                    phone: self.phone.clone(),
                    come_from: self.come_from,
                    first_name: self.first_name.clone(),
                    last_name: self.last_name.clone(),
                }
                .into(),
            )),
            CalldataYesNo::No => Ok(Jmp::Goto(Marketing {}.into())),
        }
    }
}

pub struct SelectSubscriptionsView {
    pub phone: String,
    pub come_from: ComeFrom,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[async_trait]
impl View for SelectSubscriptionsView {
    fn name(&self) -> &'static str {
        "SelectSubscriptionsView"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::SellSubscription)?;
        let text = "Выберите абонемент";
        let mut keymap = InlineKeyboardMarkup::default();
        let subscriptions = ctx.ledger.subscriptions.get_all(&mut ctx.session).await?;
        for subscription in &subscriptions {
            keymap = keymap.append_row(vec![SelectSubscriptionsCallback(subscription.id.bytes())
                .button(subscription.name.clone())]);
        }
        ctx.bot.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::SellSubscription)?;
        let subscription_id: SelectSubscriptionsCallback = calldata!(data);
        let sub_id = ObjectId::from_bytes(subscription_id.0);
        Ok(Jmp::Next(
            ConfirmSellSubscription {
                phone: self.phone.clone(),
                come_from: self.come_from,
                first_name: self.first_name.clone(),
                last_name: self.last_name.clone(),
                subscription_id: sub_id,
            }
            .into(),
        ))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SelectSubscriptionsCallback([u8; 12]);

pub struct ConfirmSellSubscription {
    pub phone: String,
    pub come_from: ComeFrom,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub subscription_id: ObjectId,
}

#[async_trait]
impl View for ConfirmSellSubscription {
    fn name(&self) -> &'static str {
        "ConfirmSellSubscription"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::SellSubscription)?;
        let sub = ctx
            .ledger
            .subscriptions
            .get(&mut ctx.session, self.subscription_id)
            .await?
            .ok_or_else(|| eyre::eyre!("Subscription not found"))?;

        let text = format!(
            "Все верно?:\n\
            Телефон: *{}*\n\
            Абонемент: *{}*\n\
           ",
            fmt_phone(&self.phone),
            escape(&sub.name)
        );
        let mut markup = InlineKeyboardMarkup::default();
        markup = markup.append_row(vec![
            CalldataYesNo::Yes.button("✅Да"),
            CalldataYesNo::No.button("❌Нет"),
        ]);
        ctx.bot.edit_origin(&text, markup).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::SellSubscription)?;
        match calldata!(data) {
            CalldataYesNo::Yes => {
                ctx.ledger
                    .presell_subscription(
                        &mut ctx.session,
                        self.subscription_id,
                        self.phone.clone(),
                        self.first_name.clone().unwrap_or_default(),
                        self.last_name.clone(),
                        self.come_from,
                    )
                    .await?;

                ctx.send_msg("Абонемент продан").await?;
                Ok(Jmp::Goto(Marketing {}.into()))
            }
            CalldataYesNo::No => Ok(Jmp::Goto(Marketing {}.into())),
        }
    }
}
