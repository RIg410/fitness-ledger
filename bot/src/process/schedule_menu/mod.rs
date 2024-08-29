use crate::state::State;
use eyre::Result;
use ledger::Ledger;
use storage::user::User;
use teloxide::{
    types::{CallbackQuery, Message},
    Bot,
};

#[derive(Clone, Debug)]
pub enum ScheduleState {
    Start,
}

pub async fn go_to_schedule(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    msg: &Message,
) -> Result<Option<State>> {
    

    todo!()
}

pub async fn handle_message(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    message: &Message,
    state: ScheduleState,
) -> Result<Option<State>> {
    println!("handle_message");
    todo!()
}

pub async fn handle_callback(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    q: &CallbackQuery,
    state: ScheduleState,
) -> Result<Option<State>> {
    println!("handle_callback");
    todo!()
}
