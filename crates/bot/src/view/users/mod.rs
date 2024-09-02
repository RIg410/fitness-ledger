use super::{profile::user_type, View};
use crate::callback_data::Calldata as _;
use crate::view::users::profile::UserProfile;
use crate::{context::Context, state::Widget};
use async_trait::async_trait;
use model::rights::Rule;
use model::user::User;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};
pub mod profile;
pub mod rights;

pub const LIMIT: u64 = 7;

#[derive(Default, Clone)]
pub struct UsersView {
    query: Query,
}

impl UsersView {
    pub fn new(query: Query) -> UsersView {
        UsersView { query }
    }
}

#[async_trait]
impl View for UsersView {
    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let count = ctx.ledger.user_count().await?;
        let users = ctx
            .ledger
            .find_users(&self.query.query, self.query.offset, LIMIT)
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

        self.query = Query { query, offset: 0 };
        self.show(ctx).await?;
        Ok(None)
    }

    async fn handle_callback(
        &mut self,
        ctx: &mut Context,
        data: &str,
    ) -> Result<Option<Widget>, eyre::Error> {
        ctx.ensure(Rule::ViewUsers)?;

        match Callback::from_data(data)? {
            Callback::Next => {
                self.query.offset += LIMIT;
                self.show(ctx).await?;
            }
            Callback::Prev => {
                self.query.offset = self.query.offset.saturating_sub(LIMIT);
                self.show(ctx).await?;
            }
            Callback::Select(user_id) => {
                let user_view = Box::new(UserProfile::new(user_id, Some(Box::new(self.clone()))));
                return Ok(Some(user_view));
            }
        }

        Ok(None)
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

    let mut markup = InlineKeyboardMarkup::default();

    for user in users {
        markup = markup.append_row(vec![make_button(user)]);
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
        markup = markup.append_row(raw);
    }
    (msg, markup)
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
