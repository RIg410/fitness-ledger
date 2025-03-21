use std::sync::Arc;

use super::build_context;
use crate::{
    context::Context, err::handle_result, state::{State, StateHolder}, widget::Widget, BACK_NAME, ERROR
};
use env::Env;
use ledger::Ledger;
use log::error;
use teloxide::{
    prelude::{Requester as _, ResponseResult},
    types::CallbackQuery,
    utils::markdown::escape,
    Bot,
};

pub async fn callback_handler(
    bot: Bot,
    env: Env,
    q: CallbackQuery,
    ledger: Arc<Ledger>,
    state_holder: StateHolder,
    system_handler: impl Fn() -> Widget,
) -> ResponseResult<()> {
    let (mut ctx, widget) = if let Some(original_message) = &q.message {
        let chat_id = original_message.chat().id;
        match build_context(bot, ledger, chat_id, &state_holder, env).await {
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

    let result = match inner_callback_handler(
        &mut ctx,
        widget,
        q.data.unwrap_or_default(),
        &state_holder,
        system_handler,
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(err) => {
            error!("Failed to handle message: {:#}", err);
            if ctx.is_admin() {
                if let Err(err) = ctx
                    .send_msg(&escape(&format!("Failed to handle message: {:#}", err)))
                    .await
                {
                    error!("send message error :{:#}", err);
                }
            } else if let Err(err) = ctx.send_msg(&escape(ERROR)).await {
                error!("send message error :{:#}", err);
            }
            Ok(())
        }
    };
    ctx.bot.answer_callback_query(q.id).await?;
    result
}

async fn inner_callback_handler(
    ctx: &mut Context,
    widget: Option<Widget>,
    data: String,
    state_holder: &StateHolder,
    system_handler: impl Fn() -> Widget,
) -> Result<(), eyre::Error> {
    if !ctx.is_active() {
        ctx.send_msg("Ваш аккаунт заблокирован").await?;
        return Ok(());
    }

    let has_widget = widget.is_some();
    let mut widget = widget.unwrap_or_else(&system_handler);

    if !has_widget {
        widget.show(ctx).await?;
    }

    ctx.set_system_go_back(!widget.is_back_main_view() && !widget.main_view());

    let widget = if data.starts_with("/") {
        match data.as_str() {
            BACK_NAME => {
                if let Some(mut back) = widget.take_back() {
                    back.show(ctx).await?;
                    back
                } else {
                    system_handler()
                }
            }
            _ => system_handler(),
        }
    } else {
        widget
    };

    let mut widget = if !ctx.is_real_user && !widget.allow_unsigned_user() {
        let mut handler = system_handler();
        handler.show(ctx).await?;
        handler
    } else {
        widget
    };
    
    let result = widget.handle_callback(ctx, data.as_str()).await;
    let mut new_widget = match handle_result(ctx, result).await? {
        crate::widget::Jmp::Next(mut new_widget) => {
            new_widget.set_back(widget);
            new_widget
        }
        crate::widget::Jmp::Stay => widget,
        crate::widget::Jmp::Back => widget.take_back().unwrap_or_else(&system_handler),
        crate::widget::Jmp::Home => system_handler(),
        crate::widget::Jmp::Goto(widget) => widget,
        crate::widget::Jmp::BackSteps(steps) => {
            let mut back = widget;
            for _ in 0..steps {
                back = back.take_back().unwrap_or_else(&system_handler)
            }
            back
        }
    };
    ctx.set_system_go_back(!new_widget.is_back_main_view());

    new_widget.show(ctx).await?;

    state_holder.set_state(
        ctx.chat_id(),
        State {
            view: Some(new_widget),
            origin: Some(ctx.origin().clone()),
        },
    );
    Ok(())
}
