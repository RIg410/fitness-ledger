use bot_core::context::Context;
use eyre::Error;
use log::warn;
use model::subscription::SubscriptionType;
use teloxide::utils::markdown::escape;

pub async fn fmt_subscription_type(
    ctx: &mut Context,
    tp: &SubscriptionType,
    for_user: bool,
) -> Result<String, Error> {
    Ok(match tp {
        SubscriptionType::Group { program_filter } => {
            let mut msg = "*Групповые занятия:*".to_string();

            for program_id in program_filter {
                let program = ctx
                    .ledger
                    .programs
                    .get_by_id(&mut ctx.session, *program_id)
                    .await?;
                if let Some(program) = program {
                    if program.visible || !for_user {
                        msg.push_str(&format!("\n \\- _{}_", escape(&program.name)));
                    }
                } else {
                    warn!("Program not found: {:?}", program_id);
                }
            }
            msg
        }
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
