use crate::state::AppState;

pub enum Transition {
    Push(AppState),
    Pop,
    To(AppState),
}
