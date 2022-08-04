use app_event::event::Event;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use input::keybind::KeyBind;

pub fn app_inputs() -> Vec<KeyBind<Event>> {
    vec![
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            action: Event::Exit,
        },
        KeyBind {
            event: KeyEvent::new(
                KeyCode::Char('C'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ),
            action: Event::Exit,
        },
    ]
}
