use crate::{
    event::Event,
    state::{MoveDirection, MoveEnd},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use input::keybind::KeyBind;

#[allow(clippy::too_many_lines)]
pub fn home_inputs() -> Vec<KeyBind<Event>> {
    [
        exit(),
        cycle_tabs(),
        handle_highlights(),
        select(),
        cycle_panel(),
    ]
    .concat()
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

pub fn choice_inputs() -> Vec<KeyBind<Event>> {
    [handle_highlights(), select()].concat()
}

pub fn user_input_inputs() -> Vec<KeyBind<Event>> {
    select()
}

#[inline]
pub const fn timed_info_inputs() -> Vec<KeyBind<Event>> {
    Vec::new()
}

fn exit() -> Vec<KeyBind<Event>> {
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

fn cycle_tabs() -> Vec<KeyBind<Event>> {
    vec![
        KeyBind {
            event: KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
            action: Event::CycleTab(MoveDirection::Right),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT),
            action: Event::CycleTab(MoveDirection::Left),
        },
    ]
}

fn handle_highlights() -> Vec<KeyBind<Event>> {
    vec![
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
        // First/Last Highlight
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL),
            action: Event::HomeEndHighlight(MoveEnd::First),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Up, KeyModifiers::CONTROL),
            action: Event::HomeEndHighlight(MoveEnd::First),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
            action: Event::HomeEndHighlight(MoveEnd::Last),
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Down, KeyModifiers::CONTROL),
            action: Event::HomeEndHighlight(MoveEnd::Last),
        },
    ]
}

fn select() -> Vec<KeyBind<Event>> {
    vec![
        KeyBind {
            event: KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            action: Event::Selected,
        },
        KeyBind {
            event: KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            action: Event::Selected,
        },
    ]
}

fn cycle_panel() -> Vec<KeyBind<Event>> {
    vec![
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
    ]
}
