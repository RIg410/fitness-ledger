use super::{calendar::Calendar, history::History, requests::Requests, users::Users};

pub struct Statistics {
    calendar: Calendar,
    history: History,
    users: Users,
    requests: Requests,
}

impl Statistics {
    pub(crate) fn new(
        calendar: Calendar,
        history: History,
        users: Users,
        requests: Requests,
    ) -> Self {
        Self {
            calendar,
            history,
            users,
            requests,
        }
    }
}
