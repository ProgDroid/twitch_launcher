use crate::{
    event::Event,
    state::{MoveDirection, MoveEnd},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use input::keybind::KeyBind;

#[allow(clippy::too_many_lines)]
pub fn home_inputs() -> Vec<KeyBind<Event>> {
    vec![
        // Exit
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
        // Cycle Tabs
        KeyBind {
            event: KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
            action: Event::CycleTab(MoveDirection::Right),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT),
            action: Event::CycleTab(MoveDirection::Left),
        },
        // Cycle Highlights
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE),
            action: Event::CycleHighlight(MoveDirection::Down),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('S'), KeyModifiers::SHIFT),
            action: Event::CycleHighlight(MoveDirection::Down),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            action: Event::CycleHighlight(MoveDirection::Down),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE),
            action: Event::CycleHighlight(MoveDirection::Up),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('W'), KeyModifiers::SHIFT),
            action: Event::CycleHighlight(MoveDirection::Up),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            action: Event::CycleHighlight(MoveDirection::Up),
        },
        // First/Last Highlight
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
            action: Event::HomeEndHighlight(MoveEnd::Last),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('S'), KeyModifiers::CONTROL),
            action: Event::HomeEndHighlight(MoveEnd::Last),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Down, KeyModifiers::CONTROL),
            action: Event::HomeEndHighlight(MoveEnd::Last),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL),
            action: Event::HomeEndHighlight(MoveEnd::First),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('W'), KeyModifiers::CONTROL),
            action: Event::HomeEndHighlight(MoveEnd::First),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Up, KeyModifiers::CONTROL),
            action: Event::HomeEndHighlight(MoveEnd::First),
        },
        // Select
        KeyBind {
            event: KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            action: Event::Selected,
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            action: Event::Selected,
        },
        // Cycle Panel
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            action: Event::CyclePanel(MoveDirection::Right),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('D'), KeyModifiers::SHIFT),
            action: Event::CyclePanel(MoveDirection::Right),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
            action: Event::CyclePanel(MoveDirection::Right),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
            action: Event::CyclePanel(MoveDirection::Left),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT),
            action: Event::CyclePanel(MoveDirection::Left),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            action: Event::CyclePanel(MoveDirection::Left),
        },
    ]
}

pub fn typing_inputs() -> Vec<KeyBind<Event>> {
    vec![
        KeyBind {
            event: KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            action: Event::StopTyping,
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            action: Event::Submit,
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            action: Event::DeleteChar,
        },
    ]
}

// TODO popup inputs
