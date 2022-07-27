use crate::keybind::KeyBind;
use crossterm::event::KeyEvent;
use std::fmt::Display;
use tokio::sync::mpsc::UnboundedSender;

pub struct Handler<T: Display + Clone> {
    sender: UnboundedSender<T>,
    pub inputs: Vec<KeyBind<T>>,
}

impl<T> Handler<T>
where
    T: Display + Clone,
{
    #[must_use]
    pub fn new(sender: UnboundedSender<T>, inputs: Vec<KeyBind<T>>) -> Self {
        Self { sender, inputs }
    }

    // TODO on app init, load binds from file?

    #[allow(unused_must_use)]
    pub fn handle(&self, key_event: KeyEvent) {
        if let Some(keybind) = self.inputs.iter().find(|&bind| bind.event == key_event) {
            self.sender.send(keybind.action.clone()); // TODO handle err. maybe missed inputs list to be handled next time? what should happen if receiver is closed?
        }
    }
}
