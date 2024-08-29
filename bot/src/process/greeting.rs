use std::vec;

use crate::state::State;
use eyre::{Context, Result};
use ledger::Ledger;
use log::info;
use storage::user::{User, UserName};
use teloxide::{
    payloads::SendMessageSetters as _,
    prelude::Requester as _,
    types::{ButtonRequest, Contact, KeyboardButton, KeyboardMarkup, Message, User as TgUser},
    Bot,
};

use super::main_menu::show_commands;

const GREET_START: &str =
    "Добрый день. Приветствуем вас в нашей семье.\nПожалуйста, оставьте ваш номер телефона.";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Greeting {
    #[default]
    Start,
    RequestPhone,
}

pub async fn greet(bot: Bot, msg: Message, ledger: Ledger, state: State) -> Result<Option<State>> {
    let from = if let Some(from) = &msg.from {
        from
    } else {
        return Greeting::Start.into();
    };

    if from.is_bot {
        bot.send_message(msg.chat.id, "Бот работает только с людьми.")
            .await?;
        return Greeting::Start.into();
    }

    let state = if let State::Greeting(greeting) = state {
        greeting
    } else {
        Greeting::Start
    };
    match state {
        Greeting::Start => {
            let keymap = KeyboardMarkup::new(vec![vec![
                KeyboardButton::new("📱 Отправить номер").request(ButtonRequest::Contact)
            ]]);
            bot.send_message(msg.chat.id, GREET_START)
                .reply_markup(keymap.one_time_keyboard())
                .await?;
            Greeting::RequestPhone.into()
        }
        Greeting::RequestPhone => {
            if let Some(contact) = msg.contact() {
                let user = create_user(&ledger, msg.chat.id.0, contact, from)
                    .await
                    .context("Failed to create user")?;
                bot.send_message(msg.chat.id, "Добро пожаловать!").await?;

                show_commands(&bot, &user)
                    .await
                    .context("Failed to show main menu")?;
                return Ok(Some(State::Start));
            } else {
                bot.send_message(
                    msg.chat.id,
                    "Нажмите на кнопку, чтобы отправить номер телефона.",
                )
                .await?;
                return Greeting::RequestPhone.into();
            }
        }
    }
}

pub async fn create_user(
    ledger: &Ledger,
    chat_id: i64,
    contact: &Contact,
    from: &TgUser,
) -> Result<User> {
    info!("Creating user with chat_id: {}", chat_id);
    let user = ledger.get_user_by_tg_id(&from.id.0.to_string()).await?;
    if user.is_some() {
        return Err(eyre::eyre!("User {} already exists", chat_id));
    }
    ledger
        .create_user(
            chat_id,
            from.id.0.to_string(),
            UserName {
                tg_user_name: from.username.clone(),
                first_name: from.first_name.clone(),
                last_name: from.last_name.clone(),
            },
            contact.phone_number.clone(),
        )
        .await
        .context("Failed to create user")?;
    match ledger.get_user_by_tg_id(&from.id.0.to_string()).await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(eyre::eyre!("Failed to create user")),
        Err(err) => Err(err),
    }
}
