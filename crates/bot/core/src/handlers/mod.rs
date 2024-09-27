pub mod callback;
pub mod message;

use std::sync::{atomic::AtomicBool, Arc};

use crate::{
    context::{Context, Origin},
    state::StateHolder,
    widget::Widget,
};
use eyre::Error;
use ledger::Ledger;
use model::user::User;
use teloxide::{prelude::Requester as _, types::ChatId, Bot};

async fn build_context(
    bot: Bot,
    ledger: Ledger,
    tg_id: ChatId,
    state_holder: &StateHolder,
) -> Result<(Context, Option<Widget>), (Error, Bot)> {
    let mut session = ledger
        .db
        .start_session()
        .await
        .map_err(|err| (err, bot.clone()))?;
    let (user, real) = if let Some(user) = ledger
        .users
        .get_by_tg_id(&mut session, tg_id.0)
        .await
        .map_err(|err| (err, bot.clone()))?
    {
        (user, true)
    } else {
        (User::new(tg_id.0), false)
    };
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
            is_valid: Arc::new(AtomicBool::new(true)),
        }
    };

    Ok((
        Context::new(bot, user, ledger, origin, session, real),
        state.view,
    ))
}
