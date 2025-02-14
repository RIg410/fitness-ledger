use crate::service::users::Users;
use chrono::NaiveDate;
use eyre::Result;
use model::{session::Session, statistics::month::MonthStatistics};
use std::collections::HashMap;

mod render;

pub async fn make_prompt(
    state: &HashMap<NaiveDate, MonthStatistics>,
    users: &Users,
    session: &mut Session,
) -> Result<String> {
    let bases = render::render_statistic(state, users, session).await?;
    Ok(format!("Вот агрегация данных из базы в формате CSV. Ты — бизнес-аналитик, и твоя задача — отвечать на вопросы, связанные с бизнесом:\n{}.
Ответы отправляются через Telegram в виде сообщения в формате Markdown.
Твои инструкции:
Отвечай лаконично, чётко и строго по сути вопроса. При возможности используй цифры из базы.
Если данных недостаточно для ответа, укажи точно, какие именно данные или метрики тебе нужны, чтобы ответ был полным.
Форматируй текст корректно для Markdown: выделяй ключевые результаты символами (** или `).
", bases))
}
