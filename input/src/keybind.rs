use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fmt::{Display, Formatter, Result, Write};

const SEPARATOR: &str = "+";

#[derive(Clone)]
pub struct KeyBind<T: Display + Clone> {
    pub event: KeyEvent,
    pub action: T,
}

impl<T> Display for KeyBind<T>
where
    T: Display + Clone,
{
    // TODO think about how to display multiple keys per action
    fn fmt(&self, f: &mut Formatter) -> Result {
        // for (i, trigger) in (&self.triggers).iter().enumerate() {
        //     if trigger.modifiers.contains(KeyModifiers::SHIFT) {
        //         if let KeyCode::Char(_) = trigger.code {
        //             continue;
        //         }
        //     }

        //     if i > 0 {
        //         write!(&mut bind_display, ", ")?;
        //     }

        //     if trigger.code != KeyCode::BackTab {
        //         write!(
        //             &mut bind_display,
        //             "{}",
        //             modifier_to_string(trigger.modifiers)
        //         )?;
        //     }

        //     write!(&mut bind_display, "{}", code_to_string(trigger.code))?;
        // }

        write!(f, "{}: {}", self.action, code_to_string(self.event.code))
    }
}

// #[allow(clippy::useless_let_if_seq)]
// fn modifier_to_string(modifier: KeyModifiers) -> String {
//     let mut modifier_display = String::new();
//     let mut separator = "";

//     if modifier.contains(KeyModifiers::CONTROL) {
//         write!(&mut modifier_display, "CTRL").expect("Could not write CTRL to modifier string");
//         separator = SEPARATOR;
//     }

//     if modifier.contains(KeyModifiers::ALT) {
//         write!(&mut modifier_display, "{}ALT", separator)
//             .expect("Could not write ALT to modifier string");
//         separator = SEPARATOR;
//     }

//     if modifier.contains(KeyModifiers::SHIFT) {
//         write!(&mut modifier_display, "{}SHIFT", separator)
//             .expect("Could not write SHIFT to modifier string");
//         separator = SEPARATOR;
//     }

//     if !modifier_display.is_empty() {
//         write!(&mut modifier_display, "{}", separator)
//             .expect("Could not write separator at the end of modifier string");
//     }

//     modifier_display
// }

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
        KeyCode::F(num) => format!("F{}", num),
        KeyCode::Char(c) => match c {
            ' ' => String::from("Space"),
            _ => c.to_uppercase().to_string(),
        },
        KeyCode::Null => String::from("Unknown"),
        KeyCode::Esc => String::from("Esc"),
    }
}
