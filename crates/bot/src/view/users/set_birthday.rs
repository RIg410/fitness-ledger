use super::View;
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use chrono::{Local, TimeZone as _};
use eyre::{Error, Result};
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};

pub struct SetBirthday {
    id: i64,
    go_back: Option<Widget>,
}

impl SetBirthday {
    pub fn new(id: i64, go_back: Option<Widget>) -> SetBirthday {
        SetBirthday { id, go_back }
    }
}

#[async_trait]
impl View for SetBirthday {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let msg = format!("Введите дату рождения в формате ДД\\.ММ\\.ГГГГ");
        let mut keymap = InlineKeyboardMarkup::default();

        if self.go_back.is_some() {
            keymap = keymap.append_row(Callback::Back.btn_row("⬅️"));
        }
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        let text = message.text().unwrap_or_default();
        let date = chrono::NaiveDate::parse_from_str(&text, "%d.%m.%Y")
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
                    ctx.send_err("Не удалось установить дату рождения").await?;
                }
                ctx.delete_msg(message.id).await?;
                Ok(self.go_back.take())
            }
            Err(_) => {
                ctx.send_err(&format!("Введите дату в формате ДД\\.ММ\\.ГГГГ"))
                    .await?;
                Ok(None)
            }
        }
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Option<Widget>> {
        if let Some(cb) = Callback::from_data(data) {
            match cb {
                Callback::Back => Ok(self.go_back.take()),
            }
        } else {
            Ok(None)
        }
    }

    fn take(&mut self) -> Widget {
        SetBirthday {
            id: self.id,
            go_back: self.go_back.take(),
        }
        .boxed()
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Back,
}
