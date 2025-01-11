use model::{day::StatisticsSummary, program::TrainingType, training::TrainingStatus};

pub fn fmt_training_status(
    training: TrainingStatus,
    is_processed: bool,
    is_full: bool,
    my: bool,
) -> &'static str {
    if is_processed {
        if my {
            "✔️❤️"
        } else {
            "✔️"
        }
    } else {
        match training {
            TrainingStatus::Finished => {
                if my {
                    "✅❤️"
                } else {
                    "✅"
                }
            }
            TrainingStatus::OpenToSignup { .. } => {
                if my {
                    "❤️"
                } else if is_full {
                    "🟣"
                } else {
                    "🟢"
                }
            }
            TrainingStatus::ClosedToSignup => "🟠",
            TrainingStatus::InProgress => "🔵",
            TrainingStatus::Cancelled => {
                if my {
                    "⛔💔"
                } else {
                    "⛔"
                }
            }
        }
    }
}

pub fn fmt_training_type(tp: TrainingType) -> String {
    match tp {
        TrainingType::Group { is_free } => format!(
            "Групповая тренировка {}",
            if is_free {
                "\\(бесплатная\\)"
            } else {
                ""
            }
        ),
        TrainingType::Personal { is_free } => format!(
            "Персональная тренировка {}",
            if is_free {
                "\\(бесплатная\\)"
            } else {
                ""
            }
        ),
        TrainingType::Event { is_free } => format!(
            "Событие {}",
            if is_free {
                "\\(бесплатная\\)"
            } else {
                ""
            }
        ),
    }
}


pub fn fmt_statistics_summary(stat: &StatisticsSummary) -> String {
    format!(
        "📊Заработано {}\n📊Награда инструктора {}\n📊Количество тренировок:{}\n📊Количество тренировок без клиентов:{}\n📊Клиентов:{}\n📊Средняя цена занятия:{}",
        stat.earned, 
        stat.couch_rewards,
        stat.training_count,
        stat.training_without_rewards,
        stat.clients_count,
        stat.sub_avg
    )
}