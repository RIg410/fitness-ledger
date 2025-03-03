use ai::{AiContext, AiModel};
use bson::oid::ObjectId;
use chrono::Local;
use eyre::Error;
use model::{
    history::HistoryRow, session::Session, user::{
        extension::{self, UserExtension},
        User,
    }
};
use eyre::eyre;

use super::Users;

impl Users {
    pub async fn ask_ai(
        &self,
        session: &mut Session,
        user: ObjectId,
        model: AiModel,
        prompt: String,
    ) -> Result<String, Error> {
        let mut user = self
            .store
            .get(session, user)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        self.resolve_family(session, &mut user).await?;

        let extension = self
            .store
            .get_extension(session, user.id)
            .await?;

        let history = self.logs.get_actor_logs(session, user.id, 1000, 0).await?;

        let mut ctx = AiContext::default();
        // let response = self.ai.ask(model, request_aggregation, &mut ctx).await?;
        // Ok(response.response)
        todo!()
    }
}

fn system_prompt(
    user: User,
    extension: UserExtension,
    history: Vec<HistoryRow>,
) -> Result<String, Error> {
    let mut prompt = "Ты помошник администратора. Вот данные пользователя и история его взаимодействия с ботом. Ответь на вопросы администратора на основе предоставленных данных." 
        .to_string();
    prompt.push_str(&format!("Имя: {}\n", user.name.first_name));

    if let Some(freeze) = &user.freeze {
        prompt.push_str(&format!(
            "Заморожен c {} по {}\n",
            freeze.freeze_start.with_timezone(&Local),
            freeze.freeze_end.with_timezone(&Local)
        ));
    }
    prompt.push_str(&format!("Дней для заморозки: {}\n", user.freeze_days));

    let payer = user.payer()?;

    prompt.push_str(&format!("Абонементы:\n"));
    for sub in payer.subscriptions() {
        prompt.push_str(&format!(
            "{}, осталось занятий:{}; зарезервировано:{}\n",
            sub.name, sub.balance, sub.locked_balance
        ));
    }

    Ok(prompt)
}

fn history_row_to_prompt(row: &HistoryRow) -> String {
    let dt = row.date_time.with_timezone(&Local);
    let msg = match &row.action {
        model::history::Action::BlockUser { is_active } => {
            Some(if *is_active {
                format!("Пользователь заблокирован")
            } else {
                format!("Пользователь разблокирован")
            })
        }
        model::history::Action::SignUp { start_at, name } =>  {
            Some(format!("записан на тренировку {} {}", start_at.with_timezone(&Local), name))
        },
        model::history::Action::SignOut { start_at, name } => {
            Some(format!("отписан от тренировки {} {}", start_at.with_timezone(&Local), name))
        },
        model::history::Action::SellSub {
            subscription,
            discount,
        } => {
            Some(format!("куплен абонемент {} цена: {}", subscription.name, subscription.price))
        },
        model::history::Action::PreSellSub {
            ..
        } => None,
        model::history::Action::FinalizedCanceledTraining { name, start_at } => {
            Some(format!("отменена тренировка {} {}", start_at.with_timezone(&Local), name))
        },
        model::history::Action::FinalizedTraining { name, start_at } => {
            Some(format!("Посетил тренировку {} {}", start_at.with_timezone(&Local), name))
        },
        model::history::Action::Payment {
            amount,
            description,
            date_time,
        } => None,
        model::history::Action::Deposit {
            amount,
            description,
            date_time,
        } => None,
        model::history::Action::CreateUser { name, phone } => None,
        model::history::Action::Freeze { days } => {
            Some(format!("заморожен на {} дней", days))
        },
        model::history::Action::Unfreeze {} => {
            Some(format!("разморожен"))
        },
        model::history::Action::ChangeBalance { amount } => None,
        model::history::Action::ChangeReservedBalance { amount } => None,
        model::history::Action::PayReward { amount } => None,
        model::history::Action::ExpireSubscription { subscription } => {
            todo!()
        },
        model::history::Action::BuySub {
            subscription,
            discount,
        } => {
            Some(format!("куплен абонемент {} цена: {}", subscription.name, subscription.price))
        },
        model::history::Action::RemoveFamilyMember {} => None,
        model::history::Action::AddFamilyMember {} => None,
    };
    if let Some(msg) = msg {
        format!("{} {}\n", dt.format("%d.%m.%Y %H:%M"), msg)
    } else {
        "".to_string()
    }

}
