use std::sync::Arc;

use bot_core::widget::Widget;
use chrono::Weekday;
use model::ids::WeekId;
use mongodb::bson::oid::ObjectId;



#[derive(Clone)]
pub struct TrainingListView(Arc<dyn Fn(ObjectId) -> Widget + Send + Sync + 'static>);

impl TrainingListView {
    pub fn make_widget(&self, id: ObjectId) -> Widget {
        (self.0)(id)
    }
}

#[derive(Clone)]
pub struct CalendarView(
    Arc<dyn Fn(WeekId, Option<Weekday>, Option<Filter>) -> Widget + Send + Sync + 'static>,
);

impl CalendarView {
    pub fn make_widget(
        &self,
        week_id: WeekId,
        selected_day: Option<Weekday>,
        filter: Option<Filter>,
    ) -> Widget {
        (self.0)(week_id, selected_day, filter)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Filter {
    pub proto_id: Option<ObjectId>,
}
