use super::{menu::MainMenuView, View};
use crate::{context::Context, state::Widget};
use async_trait::async_trait;
use eyre::{bail, Context as _};
use ledger::Ledger;
use log::info;
use model::user::UserName;
use mongodb::ClientSession;
use teloxide::types::{ButtonRequest, Contact, KeyboardButton, KeyboardMarkup, Message};

const GREET_START: &str =
    "Добрый день\\. Приветствуем вас в нашей семье\\.\nПожалуйста, оставьте ваш номер телефона\\.";

#[derive(Default)]
pub struct SignUpView {
    state: State,
}

#[derive(Default)]
enum State {
    #[default]
    Start,
    RequestPhone,
}

#[async_trait]
impl View for SignUpView {
    async fn show(&mut self, _: &mut Context) -> Result<(), eyre::Error> {
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Option<Widget>, eyre::Error> {
        let from = if let Some(from) = &msg.from {
            from
        } else {
            bail!("No user info");
        };

        if from.is_bot {
            ctx.send_msg("Бот работает только с людьми\\.").await?;
            return Ok(None);
        }
        match self.state {
            State::Start => {
                let keymap =
                    KeyboardMarkup::new(vec![vec![
                        KeyboardButton::new("📱 Отправить номер").request(ButtonRequest::Contact)
                    ]]);
                ctx.send_replay_markup(GREET_START, keymap).await?;
                self.state = State::RequestPhone;
                Ok(None)
            }
            State::RequestPhone => {
                if let Some(contact) = msg.contact() {
                    create_user(&ctx.ledger, msg.chat.id.0, contact, from, &mut ctx.session)
                        .await
                        .context("Failed to create user")?;
                    ctx.send_msg("Добро пожаловать\\!").await?;

                    ctx.reload_user().await?;
                    let view = Box::new(MainMenuView);
                    return Ok(Some(view));
                } else {
                    let keymap =
                        KeyboardMarkup::new(vec![vec![KeyboardButton::new("📱 Отправить номер")
                            .request(ButtonRequest::Contact)]]);
                    ctx.send_replay_markup(
                        "Нажмите на кнопку, чтобы отправить номер телефона\\.",
                        keymap,
                    )
                    .await?;
                    Ok(None)
                }
            }
        }
    }

    async fn handle_callback(
        &mut self,
        _: &mut Context,
        _: &str,
    ) -> Result<Option<Widget>, eyre::Error> {
        Ok(None)
    }

    fn allow_unsigned_user(&self) -> bool {
        true
    }
}

pub async fn create_user(
    ledger: &Ledger,
    chat_id: i64,
    contact: &Contact,
    from: &teloxide::types::User,
    session: &mut ClientSession,
) -> Result<(), eyre::Error> {
    info!("Creating user with chat_id: {}", chat_id);
    let user = ledger.users.get_by_tg_id(from.id.0 as i64).await?;
    if user.is_some() {
        return Err(eyre::eyre!("User {} already exists", chat_id));
    }
    ledger
        .users
        .create(
            session,
            chat_id,
            UserName {
                tg_user_name: from.username.clone(),
                first_name: from.first_name.clone(),
                last_name: from.last_name.clone(),
            },
            contact.phone_number.clone(),
        )
        .await
        .context("Failed to create user")?;
    Ok(())
}
