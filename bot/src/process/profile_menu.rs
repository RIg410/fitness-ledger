use crate::{
    process::{format::format_data, users_menu::user_type},
    state::State,
};
use chrono::NaiveDate;
use eyre::Result;
use ledger::{Ledger, SetDateError};
use log::warn;
use storage::user::User;
use teloxide::{
    payloads::SendMessageSetters as _,
    prelude::Requester as _,
    types::{CallbackQuery, ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
    Bot,
};

const SET_BIRTHDAY: &str = "/set_birthday";

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ProfileState {
    #[default]
    ShowStatus,
    SetDate,
}

pub async fn go_to_profile(
    bot: &Bot,
    user: &User,
    _: &Ledger,
    msg: &Message,
) -> Result<Option<State>> {
    let mut to_send = bot
        .send_message(msg.chat.id, format_user_profile(user))
        .parse_mode(teloxide::types::ParseMode::MarkdownV2);

    if user.birthday.is_none() {
        to_send = to_send.reply_markup(InlineKeyboardMarkup::default().append_row(vec![
            InlineKeyboardButton::callback("Установить дату рождения", SET_BIRTHDAY),
        ]));
    }
    to_send.await?;
    Ok(Some(State::Profile(ProfileState::ShowStatus)))
}

pub async fn handle_message(
    bot: &Bot,
    user: &User,
    ledger: &Ledger,
    msg: &Message,
    state: ProfileState,
) -> Result<Option<State>> {
    match state {
        ProfileState::ShowStatus => go_to_profile(bot, user, ledger, msg).await,
        ProfileState::SetDate => match parse_date(msg.text()) {
            Ok(date) => {
                if let Err(err) = ledger.set_user_birthday(&user.user_id, date).await {
                    match err {
                        SetDateError::UserNotFound => {
                            warn!("User {} not found", user.user_id);
                            bot.send_message(msg.chat.id, "Пользователь не найден")
                                .await?;
                            return Ok(None);
                        }
                        SetDateError::AlreadySet => {
                            warn!("User {} already has birthday", user.user_id);
                            bot.send_message(msg.chat.id, "Дата рождения уже установлена")
                                .await?;
                            return Ok(None);
                        }
                        SetDateError::Common(err) => {
                            warn!("Failed to set birthday: {:#}", err);
                            bot.send_message(msg.chat.id, "Не удалось установить дату рождения")
                                .await?;
                            return Ok(None);
                        }
                    }
                }
                go_to_profile(
                    bot,
                    &ledger.get_user_by_tg_id(&user.user_id).await?.unwrap(),
                    ledger,
                    msg,
                )
                .await
            }
            Err(err) => {
                warn!("Failed to parse date '{:?}': {:#}", msg.text(), err);
                bot.send_message(msg.chat.id, "Неверный формат даты")
                    .await?;
                Ok(Some(State::Profile(ProfileState::SetDate)))
            }
        },
    }
}

fn parse_date(date: Option<&str>) -> Result<NaiveDate> {
    let date = date.ok_or_else(|| eyre::eyre!("Date is empty"))?;
    Ok(chrono::NaiveDate::parse_from_str(date.trim(), "%d.%m.%Y")
        .map_err(|err| eyre::eyre!("Failed to parse date: {:#}", err))?)
}

pub async fn handle_callback(
    bot: &Bot,
    user: &User,
    _: &Ledger,
    q: &CallbackQuery,
    state: ProfileState,
) -> Result<Option<State>> {
    if state != ProfileState::ShowStatus {
        return Ok(Some(State::Profile(state)));
    }

    let data = if let Some(data) = q.data.as_ref() {
        data
    } else {
        return Ok(None);
    };

    match data.as_str() {
        SET_BIRTHDAY => {
            let text = "Введите дату рождения в формате ДД.ММ.ГГГГ";
            bot.send_message(ChatId(user.chat_id), text).await?;
            Ok(Some(State::Profile(ProfileState::SetDate)))
        }
        _ => Ok(None),
    }
}

pub fn format_user_profile(user: &User) -> String {
    let empty = "?".to_string();
    format!(
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
                .map(format_data)
                .unwrap_or_else(|| empty.clone())
        ),
        user.balance
    )
}
