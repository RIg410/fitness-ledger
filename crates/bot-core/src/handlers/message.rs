use super::build_context;
use crate::{
    context::Context,
    state::{State, StateHolder},
    widget::Widget,
    BACK_NAME, ERROR,
};
use ledger::Ledger;
use log::error;
use teloxide::{
    prelude::{Requester as _, ResponseResult},
    types::Message,
    utils::markdown::escape,
    Bot,
};

pub async fn message_handler(
    bot: Bot,
    msg: Message,
    ledger: Ledger,
    state_holder: StateHolder,
    system_handler: impl Fn() -> Widget,
) -> ResponseResult<()> {
    let (mut ctx, widget) = match build_context(bot, ledger, msg.chat.id, &state_holder).await {
        Ok(ctx) => ctx,
        Err((err, bot)) => {
            error!("Failed to build context: {:#}", err);
            bot.send_message(msg.chat.id, ERROR).await?;
            return Ok(());
        }
    };

    match inner_message_handler(
        &mut ctx,
        widget.unwrap_or_else(|| system_handler()),
        msg,
        &state_holder,
        system_handler,
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(err) => {
            if ctx.is_admin() {
                if let Err(err) = ctx
                    .send_msg(&escape(&format!("Failed to handle message: {:#}", err)))
                    .await
                {
                    error!("send message error :{:#}", err);
                }
            } else {
                error!("Failed to handle message: {:#}", err);
                if let Err(err) = ctx.send_msg(&escape(ERROR)).await {
                    error!("send message error :{:#}", err);
                }
            }
            Ok(())
        }
    }
}

async fn inner_message_handler(
    ctx: &mut Context,
    mut widget: Widget,
    msg: Message,
    state_holder: &StateHolder,
    system_handler: impl Fn() -> Widget,
) -> Result<(), eyre::Error> {
    if !ctx.is_active() {
        ctx.send_msg("Ваш аккаунт заблокирован").await?;
        return Ok(());
    }

    let widget = if let Some(msg) = msg.text() {
        if msg.starts_with("/") {
            match msg {
                BACK_NAME => {
                    if let Some(mut back) = widget.take_back() {
                        back.show(ctx).await?;
                        back
                    } else {
                        let mut handler = system_handler();
                        handler.show(ctx).await?;
                        handler
                    }
                }
                _ => {
                    let mut handler = system_handler();
                    handler.show(ctx).await?;
                    handler
                }
            }
        } else {
            widget
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

    let new_widget = match widget.handle_message(ctx, &msg).await? {
        crate::widget::Goto::Next(mut new_widget) => {
            new_widget.set_back(widget);
            new_widget.show(ctx).await?;
            new_widget
        }
        crate::widget::Goto::None => widget,
        crate::widget::Goto::Back => {
            let mut new_widget = widget.take_back().unwrap_or_else(|| system_handler());
            new_widget.show(ctx).await?;
            new_widget
        }
    };

    state_holder.set_state(
        ctx.chat_id(),
        State {
            view: Some(new_widget),
            origin: Some(ctx.origin()),
        },
    );
    Ok(())
}
