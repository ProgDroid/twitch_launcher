use crate::handler::Action;
use crossterm::event::KeyEvent;

#[derive(Clone)]
pub struct KeyBind<T: Action + Clone> {
    pub event: KeyEvent,
    pub action: T,
}
