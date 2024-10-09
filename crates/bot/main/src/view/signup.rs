use super::menu::MainMenuView;
use async_trait::async_trait;
use bot_core::{
    context::Context,
    widget::{Jmp, View},
};
use eyre::{bail, Context as _};
use ledger::Ledger;
use log::info;
use model::{session::Session, user::UserName};
use teloxide::types::{
    ButtonRequest, Contact, KeyboardButton, KeyboardMarkup, KeyboardRemove, Message, ReplyMarkup,
};

const GREET_START: &str =
    "Добрый день\\. Приветствуем вас в нашей семье\\.\nПожалуйста, оставьте ваш номер телефона\\.";

#[derive(Default)]
pub struct SignUpView;

#[async_trait]
impl View for SignUpView {
    fn name(&self) -> &'static str {
        "SignUpView"
    }
    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let keymap = KeyboardMarkup::new(vec![vec![
            KeyboardButton::new("📱 Отправить номер").request(ButtonRequest::Contact)
        ]]);
        ctx.send_replay_markup(
            GREET_START,
            ReplyMarkup::Keyboard(keymap.one_time_keyboard()),
        )
        .await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        let from = if let Some(from) = &msg.from {
            from
        } else {
            bail!("No user info");
        };

        if from.is_bot {
            ctx.send_msg("Бот работает только с людьми\\.").await?;
            return Ok(Jmp::Stay);
        }

        if let Some(contact) = msg.contact() {
            create_user(&ctx.ledger, msg.chat.id.0, contact, from, &mut ctx.session)
                .await
                .context("Failed to create user")?;
            ctx.send_replay_markup(
                "Добро пожаловать\\!",
                ReplyMarkup::KeyboardRemove(KeyboardRemove::new()),
            )
            .await?;
            ctx.reload_user().await?;
            let view = MainMenuView;
            view.send_self(ctx).await?;
            return Ok(view.into());
        } else {
            let keymap = KeyboardMarkup::new(vec![vec![
                KeyboardButton::new("📱 Отправить номер").request(ButtonRequest::Contact)
            ]]);
            ctx.send_replay_markup(
                "Нажмите на кнопку, чтобы отправить номер телефона\\.",
                ReplyMarkup::Keyboard(keymap.one_time_keyboard()),
            )
            .await?;
            Ok(Jmp::Stay)
        }
    }

    async fn handle_callback(&mut self, _: &mut Context, _: &str) -> Result<Jmp, eyre::Error> {
        Ok(Jmp::Stay)
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
    session: &mut Session,
) -> Result<(), eyre::Error> {
    info!("Creating user with chat_id: {}", chat_id);
    let user = ledger.users.get_by_tg_id(session, from.id.0 as i64).await?;
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
