mod process;
mod sessions;
mod state;
use eyre::Result;
use ledger::Ledger;
use log::error;
use process::main_menu::MainMenuItem;
use state::{State, StateHolder};
use teloxide::{
    prelude::{Requester as _, ResponseResult},
    types::Message,
    Bot,
};

const ERROR: &str = "Что-то пошло не так. Пожалуйста, попробуйте позже.";

pub async fn start_bot(ledger: Ledger, token: String) -> Result<()> {
    let bot = Bot::new(token);
    let state = StateHolder::default();

    bot.set_my_commands(vec![
        MainMenuItem::Profile.into(),
        MainMenuItem::Schedule.into(),
        MainMenuItem::Subscription.into(),
    ])
    .await?;
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        raw_handel(bot, msg, ledger.clone(), state.clone())
    })
    .await;
    Ok(())
}

async fn raw_handel(
    bot: Bot,
    msg: Message,
    ledger: Ledger,
    state_holder: StateHolder,
) -> ResponseResult<()> {
    dbg!(&msg);

    let state = state_holder
        .clone()
        .get_state(msg.chat.id)
        .unwrap_or_else(|| State::default());

    let user_id = if let Some(id) = msg.from.as_ref().map(|user| user.id.0.to_string()) {
        id
    } else {
        return Ok(());
    };

    let id = msg.chat.id;
    let user = match ledger.get_user_by_id(user_id).await {
        Ok(user) => user,
        Err(err) => {
            error!("Failed to get user: {:#}", err);
            bot.send_message(msg.chat.id, ERROR).await?;
            return Ok(());
        }
    };

    let new_state = if let Some(user) = user {
        process::proc(bot.clone(), msg, ledger, state, user).await
    } else {
        process::greeting::greet(bot.clone(), msg, ledger, state).await
    };

    match new_state {
        Ok(Some(new_state)) => state_holder.set_state(id, new_state),
        Ok(None) => state_holder.remove_state(id),
        Err(err) => {
            error!("Failed to process message: {:#}", err);
            bot.send_message(id, ERROR).await?;
        }
    }

    Ok(())
}
