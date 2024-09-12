use async_trait::async_trait;
use chrono::{DateTime, Local, TimeZone as _};
use ledger::SetDateError;
use log::warn;
use model::{rights::Rule, user::User};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

use crate::{callback_data::Calldata as _, context::Context, state::Widget};

use super::{menu::MainMenuItem, users::profile::render_profile_msg, View};

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
                    if let Err(err) = ctx
                        .ledger
                        .users
                        .set_user_birthday(&mut ctx.session, ctx.me.tg_id, date)
                        .await
                    {
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
        let cb = if let Some(cb) = Callback::from_data(data) {
            cb
        } else {
            return Ok(None);
        };

        match cb {
            Callback::SetDate => {
                self.wait_for_date = true;
                ctx.send_msg("Введите дату рождения в формате ДД\\.ММ\\.ГГГГ")
                    .await?;
            }
        }
        Ok(None)
    }
}

fn parse_date(date: Option<&str>) -> Result<DateTime<Local>, eyre::Error> {
    let date = date.ok_or_else(|| eyre::eyre!("Date is empty"))?;
    let date = chrono::NaiveDate::parse_from_str(date.trim(), "%d.%m.%Y")
        .map_err(|err| eyre::eyre!("Failed to parse date: {:#}", err))?;
    let date = date
        .and_hms_micro_opt(0, 0, 0, 0)
        .ok_or_else(|| eyre::eyre!("Invalid date"))?;
    Ok(Local
        .from_local_datetime(&date)
        .earliest()
        .ok_or_else(|| eyre::eyre!("Invalid date"))?)
}

pub fn render_user_profile(_: &Context, user: &User) -> (String, InlineKeyboardMarkup) {
    let msg = render_profile_msg(user);

    let mut keymap = InlineKeyboardMarkup::default();
    if user.birthday.is_none() {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "Установить дату рождения",
            Callback::SetDate.to_data(),
        )]);
    }

    keymap = keymap.append_row(vec![MainMenuItem::Home.into()]);
    (msg, keymap)
}

pub fn user_type(user: &User) -> &str {
    if user.freeze.is_some() {
        "❄️"
    } else if !user.is_active {
        "⚫"
    } else if user.rights.is_full() {
        "🔴"
    } else if user.rights.has_rule(Rule::Train) {
        "🔵"
    } else {
        "🟢"
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    SetDate,
}
