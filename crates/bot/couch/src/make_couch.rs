use async_trait::async_trait;
use bot_core::{
    context::Context,
    script::{
        list::{ListId, ListItem, StageList},
        text::StageText,
        yes_no::StageYesNo,
        Dispatch, ScriptView, Stage,
    },
    widget::Widget,
};
use eyre::Error;
use model::user::{rate::EmployeeRole, User};
use teloxide::utils::markdown::escape;

pub fn make_make_couch_view() -> Widget {
    ScriptView::new("make_couch", State::default(), Stage::list(UserList)).into()
}

#[derive(Default)]
pub struct State {
    pub query: Option<String>,
    pub user: Option<User>,
    pub description: Option<String>,
}
pub struct Confirm;

#[async_trait]
impl StageYesNo<State> for Confirm {
    async fn message(&self, _: &mut Context, state: &mut State) -> Result<String, Error> {
        let (user, desc) =
            if let (Some(user), Some(desc)) = (state.user.as_ref(), state.description.as_ref()) {
                (user, desc)
            } else {
                eyre::bail!("User, description or rate not found");
            };
        Ok(format!(
            "Подтвердите создание инструктора:\n\
            Пользователь: {} {}\n\
            Описание: {}\n\
            ",
            escape(&user.name.first_name),
            escape(&user.name.last_name.clone().unwrap_or_default()),
            escape(desc),
        ))
    }

    async fn yes(&self, ctx: &mut Context, state: &mut State) -> Result<Dispatch<State>, Error> {
        let (user, desc) =
            if let (Some(user), Some(desc)) = (state.user.as_ref(), state.description.as_ref()) {
                (user, desc)
            } else {
                eyre::bail!("User, description or rate not found");
            };

        ctx.ledger
            .users
            .make_user_employee(
                &mut ctx.session,
                user.id,
                desc.clone(),
                vec![],
                EmployeeRole::Couch,
            )
            .await?;
        ctx.send_notification("Инструктор успешно создан 🎉")
            .await;
        Ok(Dispatch::WidgetBack)
    }

    fn back(&self) -> Option<Stage<State>> {
        Some(Stage::text(CouchDescription))
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
    ) -> Result<Dispatch<State>, Error> {
        state.description = Some(query.to_string());
        Ok(Dispatch::Stage(Stage::yes_no(Confirm)))
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
        let mut users_stream = ctx
            .ledger
            .users
            .find(
                &mut ctx.session,
                &state.query.clone().unwrap_or_default(),
                offset as u64,
                limit as u64,
                Some(false),
            )
            .await?;

        let mut users = vec![];
        while let Some(user) = users_stream.next(&mut ctx.session).await {
            users.push(vec![list_item(user?)]);
        }

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
    ) -> Result<Dispatch<State>, Error> {
        let id = id.as_object_id().ok_or_else(|| eyre::eyre!("Invalid id"))?;
        let user = ctx.ledger.get_user(&mut ctx.session, id).await?;
        if user.employee.is_some() {
            ctx.send_notification("Пользователь уже инструктор").await;
            Ok(Dispatch::None)
        } else {
            state.user = Some(user);
            Ok(Dispatch::Stage(Stage::text(CouchDescription)))
        }
    }

    fn back(&self) -> Option<Stage<State>> {
        None
    }
}

fn list_item(user: User) -> ListItem {
    ListItem {
        id: user.id.into(),
        name: format!(
            "{} {}",
            user.name.first_name,
            user.name.last_name.clone().unwrap_or_default()
        ),
    }
}
