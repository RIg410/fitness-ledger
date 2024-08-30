use eyre::bail;
use ledger::Ledger;
use storage::user::{rights::Rule, User};
use teloxide::Bot;

use crate::{process::Origin, state::State};

use super::day::DayLending;

#[derive(Clone, Debug)]
pub enum PlanTrainingState {
    Lending(DayLending),
}

impl PlanTrainingState {
    pub fn origin(&self) -> &Origin {
        match self {
            PlanTrainingState::Lending(day) => &day.origin,
        }
    }
}

pub(crate) async fn go_to_plan_training(
    bot: &teloxide::Bot,
    me: &storage::user::User,
    ledger: &ledger::Ledger,
    state: PlanTrainingState,
) -> Result<Option<crate::state::State>, eyre::Error> {
    me.rights.ensure(Rule::EditSchedule)?;

    bail!("Not implemented")
}

pub(crate) async fn handle_message(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    message: &teloxide::prelude::Message,
    state: &PlanTrainingState,
) -> std::result::Result<Option<State>, eyre::Error> {
    me.rights.ensure(Rule::EditSchedule)?;

    bail!("Not implemented")
}

pub(crate) async fn handle_callback(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    q: &teloxide::prelude::CallbackQuery,
    date: PlanTrainingState,
) -> Result<Option<State>, eyre::Error> {
    me.rights.ensure(Rule::EditSchedule)?;

    bail!("Not implemented")
}
