use bot_core::context::Context;
use eyre::Error;
use model::subscription::SubscriptionType;
use teloxide::utils::markdown::escape;

pub async fn fmt_subscription_type(
    ctx: &mut Context,
    tp: &SubscriptionType,
) -> Result<String, Error> {
    Ok(match tp {
        SubscriptionType::Group {} => "Групповые занятия".to_string(),
        SubscriptionType::Personal { couch_filter } => {
            if let Some(filter) = couch_filter {
                let user = ctx.ledger.users.get(&mut ctx.session, *filter).await?;
                if let Some(user) = user {
                    format!("Персональные занятия с {}", escape(&user.name.first_name))
                } else {
                    "Персональные занятия".to_string()
                }
            } else {
                "Персональные занятия".to_string()
            }
        }
    })
}
