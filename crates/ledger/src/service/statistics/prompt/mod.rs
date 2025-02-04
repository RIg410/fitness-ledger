use std::collections::HashMap;

use chrono::NaiveDate;
use eyre::Result;
use model::statistics::month::MonthStatistics;

mod render;

pub fn make_prompt(state: &HashMap<NaiveDate, MonthStatistics>) -> Result<String> {
    let bases = render::render_statistic(state)?;
    Ok(format!("Вот агрегация базы данный в формате csv. Ты бизнес аналитик и отвечаешь на вопросы касательно бизнеса:\n{}. Ответ будет отправлен через telegram в виде сообщения в формате md.", bases))

}
