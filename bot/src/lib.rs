use eyre::Result;
use ledger::Ledger;
use teloxide::{prelude::Requester as _, types::Message, Bot};

pub async fn start_bot(ledger: Ledger, token: String) -> Result<()> {
    let bot = Bot::new(token);

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
    .await;
    Ok(())
}
