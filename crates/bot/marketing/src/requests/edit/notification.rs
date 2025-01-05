use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use chrono::{Local, NaiveDateTime, TimeZone as _};
use model::request::RemindLater;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};

use crate::requests::create::CalldataYesNo;

pub struct AddNotification {
    pub id: ObjectId,
}

#[async_trait]
impl View for AddNotification {
    fn name(&self) -> &'static str {
        "AddNotification"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        let text = "Напомнить позже?";
        let mut markup = InlineKeyboardMarkup::default();
        markup = markup.append_row(vec![
            CalldataYesNo::Yes.button("✅Да"),
            CalldataYesNo::No.button("❌Нет"),
        ]);
        ctx.bot.edit_origin(text, markup).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            CalldataYesNo::Yes => Ok(Jmp::Next(
                SetRemindLater {
                    id: self.id.clone(),
                }
                .into(),
            )),
            CalldataYesNo::No => {
                ctx.ledger
                    .requests
                    .add_notification(&mut ctx.session, self.id, None)
                    .await?;
                ctx.bot.send_notification("Уведомление удалено").await?;
                Ok(Jmp::Back)
            }
        }
    }
}

pub struct SetRemindLater {
    id: ObjectId,
}

#[async_trait]
impl View for SetRemindLater {
    fn name(&self) -> &'static str {
        "SetRemindLater"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        let text =
            "Напомнить через:\nВыберите вариант или ввидите дату в формате *дд\\.мм\\.гггг чч\\:мм*";
        let markup = InlineKeyboardMarkup::default();
        let mut markup = markup
            .append_row(RememberLaterCalldata::new(chrono::Duration::hours(1)).btn_row("час"));
        markup = markup
            .append_row(RememberLaterCalldata::new(chrono::Duration::hours(2)).btn_row("2 часа"));
        markup = markup
            .append_row(RememberLaterCalldata::new(chrono::Duration::hours(3)).btn_row("3 часа"));
        markup = markup
            .append_row(RememberLaterCalldata::new(chrono::Duration::days(1)).btn_row("завтра"));
        markup = markup.append_row(
            RememberLaterCalldata::new(chrono::Duration::days(2)).btn_row("послезавтра"),
        );
        markup = markup
            .append_row(RememberLaterCalldata::new(chrono::Duration::days(7)).btn_row("неделя"));
        markup = markup
            .append_row(RememberLaterCalldata::new(chrono::Duration::days(14)).btn_row("2 недели"));
        markup = markup
            .append_row(RememberLaterCalldata::new(chrono::Duration::days(30)).btn_row("месяц"));
        markup = markup
            .append_row(RememberLaterCalldata::new(chrono::Duration::days(90)).btn_row("3 месяца"));
        ctx.bot.edit_origin(text, markup).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.bot.delete_msg(message.id).await?;

        let text = if let Some(text) = message.text() {
            text
        } else {
            return Ok(Jmp::Stay);
        };

        let dt = NaiveDateTime::parse_from_str(text, "%d.%m.%Y %H:%M")
            .ok()
            .and_then(|dt| Local.from_local_datetime(&dt).single());
        if let Some(dt) = dt {
            ctx.ledger
                .requests
                .add_notification(
                    &mut ctx.session,
                    self.id,
                    Some(RemindLater {
                        date_time: dt.with_timezone(&chrono::Utc),
                        user_id: ctx.me.id,
                    }),
                )
                .await?;
            ctx.bot.send_notification("Уведомление установлено").await?;
            Ok(Jmp::Back2)
        } else {
            ctx.bot
                .send_notification("Введите корректную дату *дд\\.мм\\.гггг чч\\:мм*")
                .await?;
            Ok(Jmp::Stay)
        }
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        let remind_later: RememberLaterCalldata = calldata!(data);
        let now = chrono::Local::now();
        let remind_later = now + chrono::Duration::seconds(remind_later.remind_later as i64);

        ctx.ledger
            .requests
            .add_notification(
                &mut ctx.session,
                self.id,
                Some(RemindLater {
                    date_time: remind_later.with_timezone(&chrono::Utc),
                    user_id: ctx.me.id,
                }),
            )
            .await?;
        ctx.bot.send_notification("Уведомление установлено").await?;
        Ok(Jmp::Back2)
    }
}

#[derive(Serialize, Deserialize)]
pub struct RememberLaterCalldata {
    remind_later: u64,
}

impl RememberLaterCalldata {
    pub fn new(duration: chrono::Duration) -> Self {
        Self {
            remind_later: duration.num_seconds() as u64,
        }
    }
}
