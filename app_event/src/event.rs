use std::fmt::{Display, Formatter, Result};
use ui::theme::Theme;

#[derive(Clone)]
pub enum Event {
    Exit,
    SetTheme(Theme),
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Self::Exit => write!(f, "Exit App"),
            Self::SetTheme(_) => write!(f, "Set Theme"),
        }
    }
}
