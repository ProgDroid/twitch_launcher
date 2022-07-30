use crate::keybind::KeyBind;
use crossterm::event::KeyEvent;
use std::fmt::Display;

pub struct Handler<T: Display + Clone> {
    pub inputs: Vec<KeyBind<T>>,
}

impl<T> Handler<T>
where
    T: Display + Clone,
{
    #[must_use]
    pub fn new(inputs: Vec<KeyBind<T>>) -> Self {
        Self { inputs }
    }

    // TODO on app init, load binds from file?

    #[must_use]
    pub fn handle(&self, key_event: KeyEvent) -> Option<T> {
        if let Some(keybind) = self.inputs.iter().find(|&bind| bind.event == key_event) {
            return Some(keybind.action.clone());
        }

        None
    }
}
