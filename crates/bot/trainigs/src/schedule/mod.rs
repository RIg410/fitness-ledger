use ledger::service::calendar::TimeSlotCollision;
use teloxide::utils::markdown::escape;

pub mod group;
pub mod personal;
pub mod rent;


pub fn render_time_slot_collision(collision: &TimeSlotCollision) -> String {
    format!(
        "Это время уже занято другой тренировкой:\n*{}*\n\nДата:*{}*",
        escape(&collision.name),
        collision.get_slot().start_at().format("%d\\.%m %H:%M")
    )
}
