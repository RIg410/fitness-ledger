use model::training::TrainingStatus;

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
