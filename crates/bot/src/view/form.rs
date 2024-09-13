use crate::state::Widget;

pub struct Form<T: Default> {
    state: Option<T>,
    back: Option<Widget>,
    actions: Vec<Dialog>,
}

pub enum Dialog {
    Text {},
    Callback {},
}

