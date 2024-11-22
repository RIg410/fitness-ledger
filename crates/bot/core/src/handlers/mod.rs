pub mod callback;
pub mod message;

use std::sync::Arc;

use crate::{
    bot::{Origin, TgBot},
    context::Context,
    state::StateHolder,
    widget::Widget,
};
use env::Env;
use eyre::Error;
use ledger::Ledger;
use model::user::User;
use teloxide::{prelude::Requester as _, types::ChatId, Bot};

async fn build_context(
    bot: Bot,
    ledger: Arc<Ledger>,
    tg_id: ChatId,
    state_holder: &StateHolder,
    env: Env,
) -> Result<(Context, Option<Widget>), (Error, Bot)> {
    let mut session = ledger
        .db
        .start_session()
        .await
        .map_err(|err| (err, bot.clone()))?;
    let (mut user, real) = if let Some(user) = ledger
        .users
        .get_by_tg_id(&mut session, tg_id.0)
        .await
        .map_err(|err| (err, bot.clone()))?
    {
        (user, true)
    } else {
        (User::with_tg_id(tg_id.0), false)
    };
    ledger
        .users
        .resolve_family(&mut session, &mut user)
        .await
        .map_err(|err| (err, bot.clone()))?;
    session.set_actor(user.id);
    let state = state_holder.get_state(tg_id).unwrap_or_default();

    let origin = if let Some(origin) = state.origin {
        origin
    } else {
        let id = bot
            .send_message(tg_id, ".")
            .await
            .map_err(|err| (err.into(), bot.clone()))?
            .id;
        Origin {
            chat_id: tg_id,
            message_id: id,
            tkn: state_holder.get_token(tg_id),
        }
    };

    let tg_bot = TgBot::new(bot, state_holder.tokens(), origin, env);

    Ok((
        Context::new(tg_bot, user, ledger, session, real),
        state.view,
    ))
}
