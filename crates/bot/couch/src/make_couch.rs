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
use bot_viewer::user::fmt_group_rate;
use eyre::Error;
use group_rate::SetGroupRewardType;
use model::{
    couch::{GroupRate, PersonalRate},
    decimal::Decimal,
    user::User,
};
use teloxide::utils::markdown::escape;

pub fn make_make_couch_view() -> Widget {
    ScriptView::new("make_couch", State::default(), Stage::list(UserList)).into()
}

#[derive(Default)]
pub struct State {
    pub query: Option<String>,
    pub user: Option<User>,
    pub description: Option<String>,
    pub group_rate: Option<GroupRate>,
    pub personal_rate: Option<PersonalRate>,
}

mod group_rate {
    use super::{CouchDescription, PersonalInterest, State};
    use async_trait::async_trait;
    use bot_core::{
        context::Context,
        script::{
            list::{ListId, ListItem, StageList},
            text::StageText,
            Dispatch, Stage,
        },
    };
    use chrono::{Local, NaiveDate, TimeZone as _, Utc};
    use eyre::Error;
    use model::{couch::GroupRate, decimal::Decimal};

    pub struct SetGroupRewardType;

    #[async_trait]
    impl StageList<State> for SetGroupRewardType {
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

            Ok(("Выберите тип вознаграждения 💰".to_string(), rewards))
        }

        async fn select(
            &self,
            _: &mut Context,
            state: &mut State,
            id: ListId,
        ) -> Result<Dispatch<State>, Error> {
            let id = id.as_i64().ok_or_else(|| eyre::eyre!("Invalid id"))?;
            Ok(match id {
                0 => {
                    state.group_rate = Some(GroupRate::None);
                    Dispatch::Stage(Stage::text(PersonalInterest))
                }
                1 => Dispatch::Stage(Stage::text(FixedRate)),
                2 => Dispatch::Stage(Stage::text(ClientRate)),
                _ => return Ok(Dispatch::None),
            })
        }

        fn back(&self) -> Option<Stage<State>> {
            Some(Stage::text(CouchDescription))
        }
    }

    pub struct FixedRate;

    #[async_trait]
    impl StageText<State> for FixedRate {
        async fn message(&self, _: &mut Context, _: &mut State) -> Result<String, eyre::Error> {
            Ok("Введите фиксированное вознаграждение 💵".to_string())
        }

        async fn handle_text(
            &self,
            _: &mut Context,
            state: &mut State,
            query: &str,
        ) -> Result<Dispatch<State>, Error> {
            let rate = query.parse::<Decimal>()?;
            let rate = GroupRate::FixedMonthly {
                rate,
                next_reward: Utc::now(),
            };
            state.group_rate = Some(rate);
            Ok(Dispatch::Stage(Stage::text(FixedRateNextReward)))
        }

        fn back(&self) -> Option<Stage<State>> {
            Some(Stage::list(SetGroupRewardType))
        }
    }

    struct ClientRate;

    #[async_trait]
    impl StageText<State> for ClientRate {
        async fn message(&self, _: &mut Context, _: &mut State) -> Result<String, eyre::Error> {
            Ok("Введите минимальное вознаграждение 💵".to_string())
        }

        async fn handle_text(
            &self,
            _: &mut Context,
            state: &mut State,
            query: &str,
        ) -> Result<Dispatch<State>, Error> {
            let min = query.parse::<Decimal>()?;

            state.group_rate = Some(GroupRate::PerClient {
                min,
                per_client: Decimal::zero(),
            });
            Ok(Dispatch::Stage(Stage::text(ClientRatePerClient)))
        }

        fn back(&self) -> Option<Stage<State>> {
            Some(Stage::list(SetGroupRewardType))
        }
    }

    struct ClientRatePerClient;

    #[async_trait]
    impl StageText<State> for ClientRatePerClient {
        async fn message(&self, _: &mut Context, _: &mut State) -> Result<String, eyre::Error> {
            Ok("Введите вознаграждение за клиента 💵".to_string())
        }

        async fn handle_text(
            &self,
            _: &mut Context,
            state: &mut State,
            query: &str,
        ) -> Result<Dispatch<State>, Error> {
            let per_client = query.parse::<Decimal>()?;

            if let Some(GroupRate::PerClient { min, .. }) = state.group_rate.as_mut() {
                state.group_rate = Some(GroupRate::PerClient {
                    min: *min,
                    per_client,
                });
            } else {
                eyre::bail!("Rate not found");
            }
            Ok(Dispatch::Stage(Stage::text(PersonalInterest)))
        }

        fn back(&self) -> Option<Stage<State>> {
            Some(Stage::text(ClientRate))
        }
    }

    struct FixedRateNextReward;

    #[async_trait]
    impl StageText<State> for FixedRateNextReward {
        async fn message(&self, _: &mut Context, _: &mut State) -> Result<String, eyre::Error> {
            Ok("Введите дату следующего вознаграждения 📅\nd\\.m\\.Y".to_string())
        }

        async fn handle_text(
            &self,
            _: &mut Context,
            state: &mut State,
            query: &str,
        ) -> Result<Dispatch<State>, Error> {
            let next_reward = NaiveDate::parse_from_str(query, "%d.%m.%Y")?;
            let next_reward = Local
                .from_local_datetime(
                    &next_reward
                        .and_hms_opt(0, 0, 0)
                        .ok_or_else(|| eyre::eyre!("Invalid time"))?,
                )
                .earliest()
                .ok_or_else(|| eyre::eyre!("Invalid time"))?;

            let rate = if let Some(GroupRate::FixedMonthly { rate, .. }) = state.group_rate.as_mut()
            {
                GroupRate::FixedMonthly {
                    rate: *rate,
                    next_reward: next_reward.with_timezone(&Utc),
                }
            } else {
                eyre::bail!("Rate not found");
            };
            state.group_rate = Some(rate);
            Ok(Dispatch::Stage(Stage::text(PersonalInterest)))
        }

        fn back(&self) -> Option<Stage<State>> {
            Some(Stage::list(SetGroupRewardType))
        }
    }
}

struct PersonalInterest;

#[async_trait]
impl StageText<State> for PersonalInterest {
    async fn message(&self, _: &mut Context, _: &mut State) -> Result<String, eyre::Error> {
        Ok("Введите процент вознаграждения от пересональной тренировки 💵".to_string())
    }

    async fn handle_text(
        &self,
        _: &mut Context,
        state: &mut State,
        query: &str,
    ) -> Result<Dispatch<State>, Error> {
        let couch_interest = query.parse::<Decimal>()?;
        state.personal_rate = Some(PersonalRate { couch_interest });
        Ok(Dispatch::Stage(Stage::yes_no(Confirm)))
    }

    fn back(&self) -> Option<Stage<State>> {
        Some(Stage::list(SetGroupRewardType))
    }
}

pub struct Confirm;

#[async_trait]
impl StageYesNo<State> for Confirm {
    async fn message(&self, _: &mut Context, state: &mut State) -> Result<String, Error> {
        let (user, desc, rate) = if let (Some(user), Some(desc), Some(rate)) = (
            state.user.as_ref(),
            state.description.as_ref(),
            state.group_rate.as_ref(),
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
            escape(desc),
            fmt_group_rate(rate)
        ))
    }

    async fn yes(&self, ctx: &mut Context, state: &mut State) -> Result<Dispatch<State>, Error> {
        let (user, desc, rate, personal_rate) =
            if let (Some(user), Some(desc), Some(rate), Some(personal_rate)) = (
                state.user.as_ref(),
                state.description.as_ref(),
                state.group_rate.as_ref(),
                state.personal_rate.as_ref(),
            ) {
                (user, desc, rate, personal_rate)
            } else {
                eyre::bail!("User, description or rate not found");
            };

        ctx.ledger
            .users
            .make_user_couch(
                &mut ctx.session,
                user.id,
                desc.clone(),
                rate.clone(),
                personal_rate.clone(),
            )
            .await?;
        ctx.send_notification("Инструктор успешно создан 🎉")
            .await?;
        Ok(Dispatch::WidgetBack)
    }

    fn back(&self) -> Option<Stage<State>> {
        Some(Stage::list(SetGroupRewardType))
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
        Ok(Dispatch::Stage(Stage::list(SetGroupRewardType)))
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
        if user.couch.is_some() {
            ctx.send_notification("Пользователь уже инструктор").await?;
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
