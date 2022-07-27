use crate::event::Event;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use input::keybind::KeyBind;

pub fn home_inputs() -> Vec<KeyBind<Event>> {
    vec![
        KeyBind {
            event: KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            action: Event::Exited,
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
            action: Event::Exited,
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::SHIFT),
            action: Event::Exited,
        },
    ]
}
