use std::vec;

use async_trait::async_trait;
use eyre::Error;
use model::{couch::Rate, user::User};
use teloxide::utils::markdown::escape;

use crate::{
    context::Context,
    state::Widget,
    view::{
        script::{
            list::{ListId, ListItem, StageList},
            text::StageText,
            ScriptView, Stage, StageYesNo,
        },
        users::profile::render_rate,
        View,
    },
};

pub fn make_make_couch_view(go_back: Widget) -> Widget {
    ScriptView::new(State::default(), Stage::list(UserList), go_back).boxed()
}

#[derive(Default)]
pub struct State {
    pub query: Option<String>,
    pub user: Option<User>,
    pub description: Option<String>,
    pub reward_rate: Option<Rate>,
}

pub struct SetRewardType;

#[async_trait]
impl StageList<State> for SetRewardType {
    async fn message(
        &self,
        _: &mut Context,
        _: &mut State,
        _: usize,
        _: usize,
    ) -> Result<(String, Vec<Vec<ListItem>>), Error> {
        let mut rewards = vec![];

        rewards.push(vec![ListItem {
            id: ListId::I64(0),
            name: "Без вознаграждения 🚫".to_string(),
        }]);
        rewards.push(vec![ListItem {
            id: ListId::I64(1),
            name: "Фиксированное вознаграждение 💵".to_string(),
        }]);
        rewards.push(vec![ListItem {
            id: ListId::I64(2),
            name: "По клиентам 👥".to_string(),
        }]);

        Ok((format!("Выберите тип вознаграждения 💰"), rewards))
    }

    async fn select(
        &self,
        ctx: &mut Context,
        state: &mut State,
        id: ListId,
    ) -> Result<Option<Stage<State>>, Error> {
        let id = id.as_i64().ok_or_else(|| eyre::eyre!("Invalid id"))?;
        Ok(match id {
            0 => {
                state.reward_rate = Some(Rate::None);
                Some(Stage::yes_no(Confirm))
            }
            1 => todo!("Fixed rate"),
            2 => todo!("Rate by clients"),
            _ => return Ok(None),
        })
    }

    fn back(&self) -> Option<Stage<State>> {
        Some(Stage::text(CouchDescription))
    }
}

pub struct Confirm;

#[async_trait]
impl StageYesNo<State> for Confirm {
    async fn message(&self, _: &mut Context, state: &mut State) -> Result<String, Error> {
        let (user, desc, rate) = if let (Some(user), Some(desc), Some(rate)) = (
            state.user.as_ref(),
            state.description.as_ref(),
            state.reward_rate.as_ref(),
        ) {
            (user, desc, rate)
        } else {
            eyre::bail!("User, description or rate not found");
        };
        Ok(format!(
            "Подтвердите создание инструктора:\n\
            Пользователь: {} {}\n\
            Описание: {}\n\
            Вознаграждение: {}\n\
            ",
            escape(&user.name.first_name),
            escape(&user.name.last_name.clone().unwrap_or_default()),
            escape(&desc),
            render_rate(rate)
        ))
    }

    async fn yes(
        &self,
        ctx: &mut Context,
        state: &mut State,
    ) -> Result<Option<Stage<State>>, Error> {
        let (user, desc, rate) = if let (Some(user), Some(desc), Some(rate)) = (
            state.user.as_ref(),
            state.description.as_ref(),
            state.reward_rate.as_ref(),
        ) {
            (user, desc, rate)
        } else {
            eyre::bail!("User, description or rate not found");
        };
        ctx.ledger
            .users
            .make_user_instructor(&mut ctx.session, user.tg_id, desc.clone(), rate.clone())
            .await?;

        Ok(None)
    }

    fn back(&self) -> Option<Stage<State>> {
        Some(Stage::list(SetRewardType))
    }
}

pub struct CouchDescription;

#[async_trait]
impl StageText<State> for CouchDescription {
    async fn message(&self, _: &mut Context, _: &mut State) -> Result<String, eyre::Error> {
        Ok("Введите описание 📝".to_string())
    }

    async fn handle_text(
        &self,
        _: &mut Context,
        state: &mut State,
        query: &str,
    ) -> Result<Option<Stage<State>>, Error> {
        state.description = Some(query.to_string());
        Ok(Some(Stage::list(SetRewardType)))
    }

    fn back(&self) -> Option<Stage<State>> {
        Some(Stage::list(UserList))
    }
}

pub struct UserList;

#[async_trait]
impl StageList<State> for UserList {
    async fn message(
        &self,
        ctx: &mut Context,
        state: &mut State,
        limit: usize,
        offset: usize,
    ) -> Result<(String, Vec<Vec<ListItem>>), Error> {
        let users = ctx
            .ledger
            .users
            .find(
                &mut ctx.session,
                &state.query.clone().unwrap_or_default(),
                offset as u64,
                limit as u64,
            )
            .await?
            .into_iter()
            .map(|u| vec![ListItem::from(u)])
            .collect();
        Ok((
            format!(
                "Введите поисковый запрос, что бы выбрать пользователя\\.\n\\'{}\\'",
                state.query.clone().unwrap_or_default()
            ),
            users,
        ))
    }

    async fn query(&self, _: &mut Context, state: &mut State, query: &str) -> Result<(), Error> {
        state.query = Some(query.to_string());
        Ok(())
    }

    async fn select(
        &self,
        ctx: &mut Context,
        state: &mut State,
        id: ListId,
    ) -> Result<Option<Stage<State>>, Error> {
        let id = id.as_object_id().ok_or_else(|| eyre::eyre!("Invalid id"))?;
        let user = ctx.ledger.get_user(&mut ctx.session, id).await?;
        if user.couch.is_some() {
            ctx.send_notification("Пользователь уже инструктор").await?;
            Ok(None)
        } else {
            state.user = Some(user);
            Ok(Some(Stage::text(CouchDescription)))
        }
    }

    fn back(&self) -> Option<Stage<State>> {
        None
    }
}

impl From<User> for ListItem {
    fn from(user: User) -> Self {
        ListItem {
            id: user.id.into(),
            name: format!(
                "{} {}",
                user.name.first_name,
                user.name.last_name.clone().unwrap_or_default()
            ),
        }
    }
}
