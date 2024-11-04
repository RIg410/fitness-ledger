mod system;
mod view;
use std::sync::Arc;

use bot_core::{
    handlers::{callback::callback_handler, message::message_handler},
    state::StateHolder,
    widget::View,
};
use env::Env;
use eyre::Result;
use ledger::Ledger;
use log::info;
use teloxide::{
    dispatching::UpdateFilterExt as _,
    dptree,
    prelude::{Dispatcher, Requester as _, ResponseResult},
    types::{CallbackQuery, InlineQuery, Message, PreCheckoutQuery, Update},
    Bot,
};
use view::menu::{MainMenuItem, MainMenuView};

#[derive(Clone)]
pub struct BotApp {
    pub bot: Bot,
    pub env: Env,
    pub state: StateHolder,
}

impl BotApp {
    pub fn new(env: Env) -> Self {
        BotApp {
            bot: Bot::new(env.tg_token()),
            state: StateHolder::default(),
            env,
        }
    }

    pub async fn start(self, ledger: Arc<Ledger>) -> Result<()> {
        let state = self.state;
        let bot = self.bot;
        bot.set_my_commands(vec![
            MainMenuItem::Home.into(),
            MainMenuItem::Profile.into(),
            MainMenuItem::Schedule.into(),
            MainMenuItem::Subscription.into(),
        ])
        .await?;

        let msg_ledger = ledger.clone();
        let env_ledger = self.env.clone();
        let msg_state = state.clone();
        let env_state = self.env.clone();

        let callback_ledger = ledger.clone();
        let callback_state = state.clone();
        let handler = dptree::entry()
            .branch(
                Update::filter_message().endpoint(move |bot: Bot, msg: Message| {
                    message_handler(
                        bot,
                        env_ledger.clone(),
                        msg,
                        msg_ledger.clone(),
                        msg_state.clone(),
                        || MainMenuView.widget(),
                    )
                }),
            )
            .branch(
                Update::filter_pre_checkout_query()
                    .endpoint(|bot: Bot, q: PreCheckoutQuery| pre_checkout_query_handler(bot, q)),
            )
            .branch(
                Update::filter_callback_query().endpoint(move |bot: Bot, q: CallbackQuery| {
                    callback_handler(
                        bot,
                        env_state.clone(),
                        q,
                        callback_ledger.clone(),
                        callback_state.clone(),
                        || MainMenuView.widget(),
                    )
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
}

async fn inline_query_handler(
    _: Bot,
    _: InlineQuery,
    _: Arc<Ledger>,
    _: StateHolder,
) -> ResponseResult<()> {
    info!("inline");
    Ok(())
}

async fn pre_checkout_query_handler(bot: Bot, q: PreCheckoutQuery) -> ResponseResult<()> {
    bot.answer_pre_checkout_query(q.id, true).await?;
    Ok(())
}
