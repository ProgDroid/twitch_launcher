use crate::keybind::KeyBind;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fmt::Write;

const BIND_SEPARATOR: &str = ", ";
const MODIFIER_SEPARATOR: &str = "+";

pub trait Action {
    fn handle(&self) -> Option<&str>;
}

pub struct Handler<T: Action + Clone> {
    pub inputs: Vec<KeyBind<T>>,
}

impl<T> Handler<T>
where
    T: Action + Clone,
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

    #[must_use]
    pub fn render(&self) -> Vec<String> {
        let mut actions: Vec<String> = Vec::new();

        let mut binds: Vec<String> = Vec::new();

        for input in &self.inputs {
            if input.event.modifiers.contains(KeyModifiers::SHIFT) {
                if let KeyCode::Char(_) = input.event.code {
                    continue;
                }
            }

            if let Some(handle) = input.action.handle() {
                let action: String = String::from(handle);

                let result = actions
                    .iter()
                    .position(|description| *description == action);

                let mut separator = BIND_SEPARATOR;

                let index = result.map_or_else(
                    || {
                        actions.push(action.clone());
                        binds.push(format!("{action}: "));
                        separator = "";
                        actions.len() - 1
                    },
                    |i| i,
                );

                binds[index].push_str(
                    format!("{}{}", separator, event_to_string(input.event).as_str()).as_str(),
                );
            }
        }

        binds
    }
}

fn event_to_string(key_event: KeyEvent) -> String {
    format!(
        "{}{}",
        if key_event.code == KeyCode::BackTab {
            String::new()
        } else {
            modifier_to_string(key_event.modifiers)
        },
        code_to_string(key_event.code)
    )
}

#[allow(clippy::useless_let_if_seq)]
fn modifier_to_string(modifier: KeyModifiers) -> String {
    let mut modifier_display = String::new();
    let mut separator = "";

    if modifier.contains(KeyModifiers::CONTROL) {
        write!(&mut modifier_display, "CTRL").expect("Could not write CTRL to modifier string");
        separator = MODIFIER_SEPARATOR;
    }

    if modifier.contains(KeyModifiers::ALT) {
        write!(&mut modifier_display, "{separator}ALT")
            .expect("Could not write ALT to modifier string");
        separator = MODIFIER_SEPARATOR;
    }

    if modifier.contains(KeyModifiers::SHIFT) {
        write!(&mut modifier_display, "{separator}SHIFT")
            .expect("Could not write SHIFT to modifier string");
        separator = MODIFIER_SEPARATOR;
    }

    if !modifier_display.is_empty() {
        write!(&mut modifier_display, "{separator}")
            .expect("Could not write separator at the end of modifier string");
    }

    modifier_display
}

fn code_to_string(code: KeyCode) -> String {
    match code {
        KeyCode::Backspace => String::from("Backspace"),
        KeyCode::Enter => String::from("Enter"),
        KeyCode::Left => String::from("Left"),
        KeyCode::Right => String::from("Right"),
        KeyCode::Up => String::from("Up"),
        KeyCode::Down => String::from("Down"),
        KeyCode::Home => String::from("Home"),
        KeyCode::End => String::from("End"),
        KeyCode::PageUp => String::from("PageUp"),
        KeyCode::PageDown => String::from("PageDown"),
        KeyCode::Tab => String::from("Tab"),
        KeyCode::BackTab => String::from("BackTab"),
        KeyCode::Delete => String::from("Delete"),
        KeyCode::Insert => String::from("Insert"),
        KeyCode::F(num) => format!("F{num}"),
        KeyCode::Char(c) => match c {
            ' ' => String::from("Space"),
            _ => c.to_uppercase().to_string(),
        },
        KeyCode::Null => String::from("Unknown"),
        KeyCode::Esc => String::from("Esc"),
        KeyCode::CapsLock => String::from("Caps Lock"),
        KeyCode::ScrollLock => String::from("Scroll Lock"),
        KeyCode::NumLock => String::from("Num Lock"),
        KeyCode::PrintScreen => String::from("Print Screen"),
        KeyCode::Menu => String::from("Menu"),
        KeyCode::Pause => String::from("Pause"),
        KeyCode::KeypadBegin => String::from("Keypad Begin"),
        KeyCode::Media(media) => format!("Media {media:?}"),
        KeyCode::Modifier(modifier) => format!("Modifier {modifier:?}"),
    }
}
