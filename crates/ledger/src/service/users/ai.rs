use ai::{AiContext, AiModel};
use bson::oid::ObjectId;
use chrono::Local;
use eyre::eyre;
use eyre::Error;
use model::decimal::Decimal;
use model::{
    history::HistoryRow,
    session::Session,
    subscription::UserSubscription,
    user::{
        extension::{self, UserExtension},
        User,
    },
};

use super::Users;

impl Users {
    pub async fn ask_ai(
        &self,
        session: &mut Session,
        user: ObjectId,
        model: AiModel,
        question: String,
    ) -> Result<String, Error> {
        let mut user = self
            .store
            .get(session, user)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        self.resolve_family(session, &mut user).await?;

        let extension = self.store.get_extension(session, user.id).await?;

        let history = self.logs.get_actor_logs(session, user.id, None, 0).await?;

        let mut ctx = AiContext::default();
        ctx.add_system_message(system_prompt(user, extension, history)?);
        let response = self.ai.ask(model, question, &mut ctx).await?;
        Ok(response.response)
    }
}

fn system_prompt(
    user: User,
    _extension: UserExtension,
    history: Vec<HistoryRow>,
) -> Result<String, Error> {
    let mut prompt = "Ты помощник администратора. Используя предоставленные данные, отвечай максимально точно и кратко, избегая лишних пояснений. Ты можешь использовать только факты из данных ниже, не делая предположений. Если данных недостаточно для ответа, сообщи об этом.
    Ты НЕ должен пытаться сделать выводы о мотивах, чувствах или фактах, не указанных в данных. Если данных недостаточно, просто сообщи: \"Информация отсутствует.\""
        .to_string();
    if let Some(freeze) = &user.freeze {
        prompt.push_str(&format!(
            "Заморожен c {} по {}\n",
            freeze.freeze_start.with_timezone(&Local),
            freeze.freeze_end.with_timezone(&Local)
        ));
    }
    prompt.push_str(&format!(
        "Доступно дней для заморозки: {}\n",
        user.freeze_days
    ));

    let payer = user.payer()?;

    prompt.push_str(&format!("Откуда пришел: {}\n", user.come_from.name()));

    prompt.push_str(&format!("Абонементы:\n"));
    for sub in payer.subscriptions() {
        prompt.push_str(&user_sub_to_prompt(sub));
    }

    prompt.push_str(&format!("История операций:\n"));
    for row in history {
        if let Some(sub) = history_row_to_prompt(&row) {
            prompt.push_str(&sub);
        }
    }
    Ok(prompt)
}

fn user_sub_to_prompt(sub: &UserSubscription) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!("Абонемент: {}\n", sub.name));
    match &sub.status {
        model::subscription::Status::Active {
            start_date,
            end_date,
        } => {
            prompt.push_str(&format!(
                "Активен с {} по {}\n",
                start_date.with_timezone(&Local),
                end_date.with_timezone(&Local)
            ));
        }
        model::subscription::Status::NotActive => {
            prompt.push_str("Абонемент не активен");
        }
    }
    prompt.push_str(&format!("Количество тренировок: {}\n", sub.balance));
    prompt.push_str(&format!(
        "Заблокированные тренировки: {}\n",
        sub.locked_balance
    ));
    if sub.unlimited {
        prompt.push_str("Абонемент безлимитный\n");
    }

    if let Some(discount) = &sub.discount {
        prompt.push_str(&format!(
            "Скидка на абонемент: {}%\n",
            *discount * Decimal::int(100)
        ));
    }

    prompt
}

fn history_row_to_prompt(row: &HistoryRow) -> Option<String> {
    let dt = row.date_time.with_timezone(&Local);
    let msg = match &row.action {
        model::history::Action::BlockUser { is_active } => Some(if *is_active {
            format!("Пользователь заблокирован")
        } else {
            format!("Пользователь разблокирован")
        }),
        model::history::Action::SignUp { start_at, name, .. } => Some(format!(
            "записан на тренировку {} {}",
            start_at.with_timezone(&Local),
            name
        )),
        model::history::Action::SignOut { start_at, name, .. } => Some(format!(
            "отписан от тренировки {} {}",
            start_at.with_timezone(&Local),
            name
        )),
        model::history::Action::SellSub {
            subscription,
            discount,
        }
        | model::history::Action::BuySub {
            subscription,
            discount,
        } => {
            if let Some(discount) = *discount {
                Some(format!(
                    "куплен абонемент {} цена: {}",
                    subscription.name,
                    subscription.price - subscription.price * discount * Decimal::int(100),
                ))
            } else {
                Some(format!(
                    "куплен абонемент {} цена: {}",
                    subscription.name, subscription.price
                ))
            }
        }
        model::history::Action::PreSellSub { .. } => None,
        model::history::Action::FinalizedCanceledTraining { .. } => None,
        model::history::Action::FinalizedTraining { name, start_at, .. } => Some(format!(
            "посетил тренировку {} {}",
            start_at.with_timezone(&Local),
            name
        )),
        model::history::Action::Payment { .. } => None,
        model::history::Action::Deposit { .. } => None,
        model::history::Action::CreateUser { .. } => None,
        model::history::Action::Freeze { days } => Some(format!("заморожен на {} дней", days)),
        model::history::Action::Unfreeze {} => Some(format!("разморожен")),
        model::history::Action::ChangeBalance { .. } => None,
        model::history::Action::ChangeReservedBalance { .. } => None,
        model::history::Action::PayReward { .. } => None,
        model::history::Action::ExpireSubscription { subscription } => Some(format!(
            "Абонемент {} истек. Сгорело: {} занятий",
            subscription.name, subscription.balance
        )),
        model::history::Action::RemoveFamilyMember {} => None,
        model::history::Action::AddFamilyMember {} => None,
        model::history::Action::ChangeSubscriptionDays { .. } => None,
    };
    msg.map(|msg| format!("{} {}\n", dt, msg))
}
