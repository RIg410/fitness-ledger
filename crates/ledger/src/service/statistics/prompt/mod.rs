use crate::service::users::Users;
use chrono::NaiveDate;
use eyre::Result;
use model::{session::Session, statistics::month::MonthStatistics};
use std::collections::HashMap;

mod render;

pub fn select_aggregation(user_request: &str) -> String {
    format!(
"Ты — аналитик в студии растяжки, который помогает определить необходимые данные для обработки запроса. У тебя есть набор заранее подготовленных агрегатов данных.
Вот доступные агрегации:
trainings_by_program - Статистика по тренировкам по программам
trainings_by_instructor - Статистика по тренировкам по инструкторам
trainings_by_room - Статистика по тренировкам по залам
trainings_by_type - Статистика по тренировкам по типам
trainings_by_weekday - Статистика по тренировкам по дням недели
trainings_by_time - Статистика по тренировкам по времени
request_aggregation - Статистика по заявкам
subscription_aggregation - Статистика по абонементам
financial_statistics - общая финансовая статистика
salary_statistics - статистика по зарплатам
marketing_financial_statistics - статистика по маркетинговым расходам
marketing_statistics - статистика по маркетингу

Пользователь задаёт тебе запрос. Твоя задача:
1) Определить, какие агрегации потребуются для ответа на запрос, а так же за какие месяцы нужно взять данные. Месяцы указываются в формате YYYY-MM-DD на первое число месяца.
2) Вернуть JSON-формат список используемых агрегаций. Только json, без лишних слов.

Пример запроса пользователя: 'Какая была тренировочная нагрузка в октябре?'
Пример ответа в JSON:
{{
    \"aggregations\": [\"trainings_by_program\", \"trainings_by_room\", \"trainings_by_type\", \"trainings_by_weekday\", \"trainings_by_time\"],
    \"months\": [\"2021-10-01\"]
}}
Дата:{}.
Вот запрос: {}.",
        chrono::Local::now().format("%Y-%m-%d"),
    user_request)
}

pub async fn make_prompt(
    state: &HashMap<NaiveDate, MonthStatistics>,
    users: &Users,
    session: &mut Session,
) -> Result<String> {
    let bases = render::render_statistic(state, users, session).await?;
    Ok(format!("Вот агрегация данных из базы в формате CSV. Ты — бизнес-аналитик, и твоя задача — отвечать на вопросы, связанные с бизнесом:\n{}.
Ответы отправляются через Telegram в виде сообщения. 
Твои инструкции:
Отвечай лаконично, чётко и строго по сути вопроса. При возможности используй цифры из базы.
Если данных недостаточно для ответа, укажи точно, какие именно данные или метрики тебе нужны, чтобы ответ был полным.
Текст будет отображаться в Telegram, поэтому избегай переносов строк и длинных предложений. Если текст слишком длинный, разбивайте его на несколько сообщений.", bases))
}
