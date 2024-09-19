use super::View;
use crate::callback_data::Calldata as _;
use crate::view::users::profile::UserProfile;
use crate::{context::Context, state::Widget};
use async_trait::async_trait;
use model::rights::Rule;
use model::user::User;
use profile::user_type;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};
pub mod freeze;
pub mod profile;
pub mod rights;
pub mod set_birthday;
pub mod set_fio;
pub mod set_phone;

pub const LIMIT: u64 = 7;

pub struct UsersView {
    query: Query,
    go_back: Option<Widget>,
}

impl UsersView {
    pub fn new(query: Query) -> UsersView {
        UsersView {
            query,
            go_back: None,
        }
    }
}

#[async_trait]
impl View for UsersView {
    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let count = ctx.ledger.users.count(&mut ctx.session).await?;
        let users = ctx
            .ledger
            .users
            .find(
                &mut ctx.session,
                &self.query.query,
                self.query.offset,
                LIMIT,
            )
            .await?;
        let (txt, markup) = render_message(count, &self.query.query, &users, self.query.offset);
        ctx.edit_origin(&txt, markup).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Option<Widget>, eyre::Error> {
        ctx.delete_msg(msg.id).await?;
        ctx.ensure(Rule::ViewUsers)?;

        let mut query = msg.text().to_owned().unwrap_or_default().to_string();
        if query.len() == 1 && !query.chars().next().unwrap().is_alphanumeric() {
            query = "".to_string();
        }

        self.query = Query {
            query: remove_non_alphanumeric(&query),
            offset: 0,
        };
        self.show(ctx).await?;
        Ok(None)
    }

    async fn handle_callback(
        &mut self,
        ctx: &mut Context,
        data: &str,
    ) -> Result<Option<Widget>, eyre::Error> {
        ctx.ensure(Rule::ViewUsers)?;

        let cb = if let Some(cb) = Callback::from_data(data) {
            cb
        } else {
            return Ok(None);
        };

        match cb {
            Callback::Next => {
                self.query.offset += LIMIT;
                self.show(ctx).await?;
            }
            Callback::Prev => {
                self.query.offset = self.query.offset.saturating_sub(LIMIT);
                self.show(ctx).await?;
            }
            Callback::Select(user_id) => {
                let user_view = UserProfile::new(user_id).boxed();
                return Ok(Some(user_view));
            }
        }

        Ok(None)
    }

    fn take(&mut self) -> Widget {
        UsersView {
            query: self.query.clone(),
            go_back: self.go_back.take(),
        }
        .boxed()
    }

    fn set_back(&mut self, back: Widget) {
        self.go_back = Some(back);
    }

    fn back(&mut self) -> Option<Widget> {
        self.go_back.take()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    pub query: String,
    pub offset: u64,
}

impl Default for Query {
    fn default() -> Self {
        Query {
            query: "".to_string(),
            offset: 0,
        }
    }
}

fn render_message(
    total_count: u64,
    query: &str,
    users: &[User],
    offset: u64,
) -> (String, InlineKeyboardMarkup) {
    let msg = format!(
        "
    🟣 Всего пользователей: _{}_
    ➖➖➖➖➖➖➖➖➖➖
    🔵 \\- Инструктор
    🟢 \\- Клиент
    🔴 \\- Администратор 

    Что бы найти пользователя, воспользуйтесь поиском\\. Введите имя, фамилию или телефон пользователя\\.\n
    Запрос: _'{}'_
    ",
        total_count,
        escape(query)
    );

    let mut keymap = InlineKeyboardMarkup::default();

    for user in users {
        keymap = keymap.append_row(vec![make_button(user)]);
    }

    let mut raw = vec![];

    if offset > 0 {
        raw.push(InlineKeyboardButton::callback(
            "⬅️",
            Callback::Prev.to_data(),
        ));
    }

    if users.len() == LIMIT as usize {
        raw.push(InlineKeyboardButton::callback(
            "➡️",
            Callback::Next.to_data(),
        ));
    }

    if raw.len() > 0 {
        keymap = keymap.append_row(raw);
    }
    (msg, keymap)
}

fn make_button(user: &User) -> InlineKeyboardButton {
    InlineKeyboardButton::callback(
        format!(
            "{}{} {}",
            user_type(user),
            user.name.first_name,
            user.name.last_name.as_ref().unwrap_or(&"".to_string())
        ),
        Callback::Select(user.tg_id).to_data(),
    )
}

#[derive(Debug, Serialize, Deserialize)]
enum Callback {
    Next,
    Prev,
    Select(i64),
}

fn remove_non_alphanumeric(input: &str) -> String {
    input.chars().filter(|c| c.is_alphanumeric()).collect()
}
