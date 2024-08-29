use ledger::Ledger;
use storage::user::User;
use teloxide::{types::{ChatId, MessageId}, Bot};
use eyre::Result;

pub async fn show_user_rights(
    bot: &Bot,
    me: &User,
    ledger: &Ledger,
    user_id: String,
    chat_id: ChatId,
    msg_id: MessageId,
) -> Result<()> {
    Ok(())
}