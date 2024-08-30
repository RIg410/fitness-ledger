use async_trait::async_trait;
use chrono::NaiveDate;
use ledger::SetDateError;
use log::warn;
use storage::user::{rights::Rule, User};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

use crate::{context::Context, state::Widget};

use super::View;

#[derive(Default)]
pub struct UserProfile {
    wait_for_date: bool,
}

#[async_trait]
impl View for UserProfile {
    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let (msg, keymap) = render_user_profile(&ctx, &ctx.me);
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Option<Widget>, eyre::Error> {
        if self.wait_for_date {
            match parse_date(msg.text()) {
                Ok(date) => {
                    if let Err(err) = ctx.ledger.set_user_birthday(ctx.me.tg_id, date).await {
                        match err {
                            SetDateError::UserNotFound => {
                                warn!("User {} not found", ctx.me.tg_id);
                                ctx.send_msg("Пользователь не найден").await?;
                                return Ok(None);
                            }
                            SetDateError::AlreadySet => {
                                warn!("User {} already has birthday", ctx.me.tg_id);
                                ctx.send_msg("Дата рождения уже установлена").await?;
                                return Ok(None);
                            }
                            SetDateError::Common(err) => {
                                warn!("Failed to set birthday: {:#}", err);
                                ctx.send_msg("Не удалось установить дату рождения").await?;
                                return Ok(None);
                            }
                        }
                    }
                    ctx.reload_user().await?;
                    ctx.update_origin_msg_id(ctx.send_msg("\\.").await?);
                    self.show(ctx).await?;
                }
                Err(err) => {
                    warn!("Failed to parse date '{:?}': {:#}", msg.text(), err);
                    ctx.send_msg("Неверный формат даты").await?;
                }
            }
        }

        ctx.delete_msg(msg.id).await?;
        Ok(None)
    }

    async fn handle_callback(
        &mut self,
        ctx: &mut Context,
        data: &str,
    ) -> Result<Option<Widget>, eyre::Error> {
        match Callback::try_from(data)? {
            Callback::SetDate => {
                self.wait_for_date = true;
                ctx.send_msg("Введите дату рождения в формате ДД\\.ММ\\.ГГГГ")
                    .await?;
            }
        }
        Ok(None)
    }
}

fn parse_date(date: Option<&str>) -> Result<NaiveDate, eyre::Error> {
    let date = date.ok_or_else(|| eyre::eyre!("Date is empty"))?;
    Ok(chrono::NaiveDate::parse_from_str(date.trim(), "%d.%m.%Y")
        .map_err(|err| eyre::eyre!("Failed to parse date: {:#}", err))?)
}

pub fn render_user_profile(_: &Context, user: &User) -> (String, InlineKeyboardMarkup) {
    let empty = "?".to_string();
    let msg = format!(
        "
    {} Пользователь : _{}_
        Имя : _{}_
        Телефон : _{}_
        Дата рождения : _{}_
        ➖➖➖➖➖➖➖➖➖➖
        *Баланс : _{}_ занятий*
        ➖➖➖➖➖➖➖➖➖➖
    ",
        user_type(user),
        escape(user.name.tg_user_name.as_ref().unwrap_or_else(|| &empty)),
        escape(&user.name.first_name),
        escape(&user.phone),
        escape(
            &user
                .birthday
                .as_ref()
                .map(|d| d.format("%d.%m.%Y").to_string())
                .unwrap_or_else(|| empty.clone())
        ),
        user.balance
    );
    let mut keymap = InlineKeyboardMarkup::default();
    if user.birthday.is_none() {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "Установить дату рождения",
            Callback::SetDate.to_data(),
        )]);
    }

    (msg, keymap)
}

pub fn user_type(user: &User) -> &str {
    if !user.is_active {
        "⚫"
    } else if user.rights.is_full() {
        "🔴"
    } else if user.rights.has_rule(Rule::Train) {
        "🔵"
    } else {
        "🟢"
    }
}

pub enum Callback {
    SetDate,
}

impl Callback {
    pub fn to_data(&self) -> String {
        match self {
            Callback::SetDate => "pcsd".to_owned(),
        }
    }
}

impl TryFrom<&str> for Callback {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pcsd" => Ok(Callback::SetDate),
            _ => Err(eyre::eyre!("failed to parse profile CB:{}", value)),
        }
    }
}
