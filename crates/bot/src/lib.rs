// pub mod format;
// mod process;
// mod sessions;
pub mod callback_data;
mod context;
mod state;
mod view;

use context::{Context, Origin};
use eyre::{Error, Result};
use ledger::Ledger;
use log::{error, info};
use model::user::User;
use state::{State, StateHolder, Widget};
use teloxide::{
    dispatching::UpdateFilterExt as _,
    dptree,
    prelude::{Dispatcher, Requester as _, ResponseResult},
    types::{CallbackQuery, ChatId, InlineQuery, Message, Update},
    utils::markdown::escape,
    Bot,
};
use view::{
    menu::{MainMenuItem, MainMenuView},
    signup::SignUpView,
    View as _,
};

const ERROR: &str = "Что-то пошло не так. Пожалуйста, попробуйте позже.";

pub async fn start_bot(ledger: Ledger, token: String) -> Result<()> {
    let bot = Bot::new(token);
    let state = StateHolder::default();

    bot.set_my_commands(vec![
        MainMenuItem::Profile.into(),
        MainMenuItem::Trainings.into(),
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
    let (mut ctx, widget, is_real_user) =
        match build_context(bot, ledger, msg.chat.id, &state_holder).await {
            Ok(ctx) => ctx,
            Err((err, bot)) => {
                error!("Failed to build context: {:#}", err);
                bot.send_message(msg.chat.id, ERROR).await?;
                return Ok(());
            }
        };

    match inner_message_handler(&mut ctx, widget, is_real_user, msg, state_holder).await {
        Ok(_) => Ok(()),
        Err(err) => {
            error!("Failed to handle message: {:#}", err);
            if let Err(err) = ctx.send_msg(&escape(ERROR)).await {
                error!("send message error :{:#}", err);
            }
            Ok(())
        }
    }
}

async fn inner_message_handler(
    ctx: &mut Context,
    widget: Option<Widget>,
    is_real_user: bool,
    msg: Message,
    state_holder: StateHolder,
) -> Result<(), eyre::Error> {
    if !ctx.is_active() {
        ctx.send_msg("Ваш аккаунт заблокирован").await?;
        return Ok(());
    }

    let view = if !is_real_user
        && !widget
            .as_ref()
            .map(|w| w.allow_unsigned_user())
            .unwrap_or_default()
    {
        let mut sign_up = SignUpView::default();
        sign_up.show(ctx).await?;
        sign_up.handle_message(ctx, &msg).await?;
        Box::new(sign_up)
    } else {
        let mut main_view = MainMenuView;
        if let Some(mut redirect) = main_view.handle_message(ctx, &msg).await? {
            redirect.show(ctx).await?;
            redirect
        } else {
            if let Some(mut widget) = widget {
                match widget.handle_message(ctx, &msg).await? {
                    Some(mut new_widget) => {
                        new_widget.show(ctx).await?;
                        new_widget
                    }
                    None => widget,
                }
            } else {
                main_view.show(ctx).await?;
                Box::new(main_view)
            }
        }
    };

    state_holder.set_state(
        ctx.chat_id(),
        State {
            view: Some(view),
            origin: Some(ctx.origin()),
        },
    );

    Ok(())
}

async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    ledger: Ledger,
    state_holder: StateHolder,
) -> ResponseResult<()> {
    let (mut ctx, widget, is_real_user) = if let Some(original_message) = &q.message {
        let chat_id = original_message.chat().id;
        match build_context(bot, ledger, chat_id, &state_holder).await {
            Ok(ctx) => ctx,
            Err((err, bot)) => {
                error!("Failed to build context: {}", err);
                bot.send_message(chat_id, ERROR).await?;
                return Ok(());
            }
        }
    } else {
        return Ok(());
    };
    match inner_callback_handler(&mut ctx, widget, is_real_user, q.data, state_holder, q.id).await {
        Ok(_) => Ok(()),
        Err(err) => {
            error!("Failed to handle message: {:#}", err);
            if let Err(err) = ctx.send_msg(&escape(ERROR)).await {
                error!("send message error :{:#}", err);
            }
            Ok(())
        }
    }
}

async fn inner_callback_handler(
    ctx: &mut Context,
    widget: Option<Widget>,
    is_real_user: bool,
    data: Option<String>,
    state_holder: StateHolder,
    id: String,
) -> Result<(), eyre::Error> {
    if !ctx.is_active() {
        ctx.send_msg("Ваш аккаунт заблокирован").await?;
        return Ok(());
    }

    let view = if !is_real_user
        && !widget
            .as_ref()
            .map(|w| w.allow_unsigned_user())
            .unwrap_or_default()
    {
        let mut sign_up = SignUpView::default();
        sign_up.show(ctx).await?;
        Box::new(sign_up)
    } else {
        let mut main_view = MainMenuView;
        if let Some(mut redirect) = main_view
            .handle_callback(
                ctx,
                data.as_ref()
                    .ok_or_else(|| eyre::eyre!("Expected callback data"))?,
            )
            .await?
        {
            redirect.show(ctx).await?;
            redirect
        } else {
            if let Some(mut widget) = widget {
                match widget
                    .handle_callback(
                        ctx,
                        data.as_ref()
                            .ok_or_else(|| eyre::eyre!("Expected callback data"))?,
                    )
                    .await?
                {
                    Some(mut new_widget) => {
                        new_widget.show(ctx).await?;
                        new_widget
                    }
                    None => widget,
                }
            } else {
                main_view.show(ctx).await?;
                Box::new(main_view)
            }
        }
    };

    state_holder.set_state(
        ctx.chat_id(),
        State {
            view: Some(view),
            origin: Some(ctx.origin()),
        },
    );

    ctx.bot.answer_callback_query(id).await?;
    Ok(())
}

async fn build_context(
    bot: Bot,
    ledger: Ledger,
    tg_id: ChatId,
    state_holder: &StateHolder,
) -> Result<(Context, Option<Widget>, bool), (Error, Bot)> {
    let mut session = ledger
        .db
        .start_session()
        .await
        .map_err(|err| (err.into(), bot.clone()))?;
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

    let state = state_holder
        .get_state(tg_id)
        .unwrap_or_else(|| State::default());

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
        }
    };

    Ok((
        Context::new(bot, user, ledger, origin, session),
        state.view,
        real,
    ))
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
