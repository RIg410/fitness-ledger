mod process;
mod sessions;
mod state;

use eyre::Result;
use ledger::Ledger;
use log::error;
use process::main_menu::MainMenuItem;
use state::{State, StateHolder};
use teloxide::{
    dispatching::UpdateFilterExt as _,
    dptree,
    prelude::{Dispatcher, Requester as _, ResponseResult},
    types::{CallbackQuery, InlineQuery, Message, Update},
    Bot, RequestError,
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

    let msg_ledger = ledger.clone();
    let msg_state = state.clone();

    let callback_ledger = ledger.clone();
    let callback_state = state.clone();
    let handler = dptree::entry()
        .branch(
            Update::filter_message().endpoint(move |bot: Bot, msg: Message| {
                message_handler(bot, msg, msg_ledger.clone(), msg_state.clone())
            }),
        )
        .branch(
            Update::filter_callback_query().endpoint(move |bot: Bot, q: CallbackQuery| {
                callback_handler(bot, q, callback_ledger.clone(), callback_state.clone())
            }),
        )
        .branch(
            Update::filter_inline_query().endpoint(move |bot: Bot, q: InlineQuery| {
                inline_query_handler(bot, q, ledger.clone(), state.clone())
            }),
        );

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    ledger: Ledger,
    state_holder: StateHolder,
) -> ResponseResult<()> {
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
    let user = match ledger.get_user_by_id(&user_id).await {
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

async fn inline_query_handler(
    _: Bot,
    q: InlineQuery,
    _: Ledger,
    _: StateHolder,
) -> Result<(), RequestError> {
    dbg!(q);
    Ok(())
}

async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    ledger: Ledger,
    state_holder: StateHolder,
) -> Result<(), RequestError> {
    if let Some(original_message) = &q.message {
        let chat_id = original_message.chat().id;
        let state = state_holder
            .clone()
            .get_state(chat_id)
            .unwrap_or_else(|| State::default());

        let user = match ledger.get_user_by_id(&q.from.id.0.to_string()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                error!("User {} not found", q.from.id.0);
                bot.send_message(chat_id, ERROR).await?;
                return Ok(());
            }
            Err(err) => {
                error!("Failed to get user: {:#}", err);
                bot.send_message(chat_id, ERROR).await?;
                return Ok(());
            }
        };

        let new_state = match state {
            State::Profile(state) => {
                process::profile_menu::handle_callback(&bot, &user, &ledger, &q, state).await
            }
            State::Users(state) => {
                process::users_menu::handle_callback(&bot, &user, &ledger, &q, state).await
            }
            State::Start | State::Greeting(_) => Ok(None),
        };

        match new_state {
            Ok(Some(new_state)) => state_holder.set_state(chat_id, new_state),
            Ok(None) => state_holder.remove_state(chat_id),
            Err(err) => {
                error!("Failed to process callback: {:#}", err);
                bot.send_message(chat_id, ERROR).await?;
            }
        }
    }
    bot.answer_callback_query(q.id).await?;
    Ok(())
}
