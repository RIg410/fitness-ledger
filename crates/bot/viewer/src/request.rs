use chrono::Local;
use model::request::Request;
use teloxide::utils::markdown::escape;

use crate::{
    day::{fmt_date, fmt_dt},
    fmt_phone,
};

pub fn fmt_request(request: &Request) -> String {
    let mut history = String::new();
    for h in &request.history {
        history.push_str(&format!(
            "\n \\- {}: {}",
            fmt_date(&h.date_time.with_timezone(&Local)),
            escape(&h.comment)
        ));
    }
    let mut remind_me = String::new();
    if let Some(remind_later) = &request.remind_later {
        remind_me = format!(
            "Напомнить: _{}_\n",
            fmt_dt(&remind_later.date_time.with_timezone(&Local))
        );
    }

    format!(
        "Заявка от {} \n*{}*\n\
        Комментарий: _{}_\n\
        Имя:  {} {}\n\
        Дата: _{}_\n{}\
        История: {}",
        fmt_phone(Some(&request.phone)),
        request.come_from.name(),
        escape(&request.comment),
        escape(request.first_name.as_deref().unwrap_or("?")),
        escape(request.last_name.as_deref().unwrap_or("?")),
        fmt_dt(&request.modified.with_timezone(&Local)),
        remind_me,
        history
    )
}
