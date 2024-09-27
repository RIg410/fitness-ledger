use async_trait::async_trait;
use bot_core::{
    context::Context,
    widget::{Jmp, View},
};
use chrono::{Local, TimeZone as _};
use eyre::{Error, Result};
use model::rights::Rule;
use teloxide::types::{InlineKeyboardMarkup, Message};

pub struct SetBirthday {
    id: i64,
}

impl SetBirthday {
    pub fn new(id: i64) -> SetBirthday {
        SetBirthday { id }
    }
}

#[async_trait]
impl View for SetBirthday {
    fn name(&self) -> &'static str {
        "SetBirthday"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let msg = "Введите дату рождения в формате ДД\\.ММ\\.ГГГГ".to_string();
        ctx.edit_origin(&msg, InlineKeyboardMarkup::default())
            .await?;
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, message: &Message) -> Result<Jmp> {
        let text = message.text().unwrap_or_default();
        let date = chrono::NaiveDate::parse_from_str(text, "%d.%m.%Y")
            .map_err(Error::new)
            .and_then(|date| {
                date.and_hms_opt(0, 0, 0)
                    .ok_or_else(|| eyre::eyre!("Invalid date"))
            })
            .and_then(|date| {
                Local
                    .from_local_datetime(&date)
                    .single()
                    .ok_or_else(|| eyre::eyre!("Invalid date"))
            });
        match date {
            Ok(date) => {
                let forced = ctx.has_right(Rule::EditUserInfo);
                let result = ctx
                    .ledger
                    .users
                    .set_user_birthday(&mut ctx.session, self.id, date, forced)
                    .await;
                if let Err(_) = result {
                    ctx.send_notification("Не удалось установить дату рождения")
                        .await?;
                }
                ctx.delete_msg(message.id).await?;
                Ok(Jmp::Back)
            }
            Err(_) => {
                ctx.send_notification("Введите дату в формате ДД\\.ММ\\.ГГГГ")
                    .await?;
                Ok(Jmp::Stay)
            }
        }
    }
}
