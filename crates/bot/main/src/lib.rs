mod system;
mod view;
use bot_core::{
    handlers::{callback::callback_handler, message::message_handler},
    state::StateHolder,
    widget::View,
};
use eyre::Result;
use ledger::Ledger;
use log::info;
use teloxide::{
    dispatching::UpdateFilterExt as _,
    dptree,
    prelude::{Dispatcher, Requester as _, ResponseResult},
    types::{CallbackQuery, InlineQuery, Message, Update},
    Bot,
};
use view::menu::{MainMenuItem, MainMenuView};

#[derive(Clone)]
pub struct BotApp {
    pub bot: Bot,
    pub state: StateHolder,
}

impl BotApp {
    pub fn new(token: String) -> Self {
        BotApp {
            bot: Bot::new(token),
            state: StateHolder::default(),
        }
    }

    pub async fn start(self, ledger: Ledger) -> Result<()> {
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
        let msg_state = state.clone();

        let callback_ledger = ledger.clone();
        let callback_state = state.clone();
        let handler = dptree::entry()
            .branch(
                Update::filter_message().endpoint(move |bot: Bot, msg: Message| {
                    message_handler(bot, msg, msg_ledger.clone(), msg_state.clone(), || {
                        MainMenuView.widget()
                    })
                }),
            )
            .branch(
                Update::filter_callback_query().endpoint(move |bot: Bot, q: CallbackQuery| {
                    callback_handler(
                        bot,
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
    _: Ledger,
    _: StateHolder,
) -> ResponseResult<()> {
    info!("inline");
    Ok(())
}
