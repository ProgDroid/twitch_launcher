use input::handler::Action;
use std::fmt::{Display, Formatter, Result};
use twitch::account::Account;
use ui::theme::Theme;

#[derive(Clone)]
pub enum Event {
    Exit,
    SetTheme(Theme),
    SetAccount(Account),
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Self::Exit => write!(f, "Exit App"),
            Self::SetTheme(_) => write!(f, "Set Theme"),
            Self::SetAccount(_) => write!(f, "Set Account"),
        }
    }
}

impl Action for Event {
    fn handle(&self) -> Option<&str> {
        None
    }
}
