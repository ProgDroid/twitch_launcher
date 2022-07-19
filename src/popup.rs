#[derive(Clone)]
pub enum Popups {
    Choice {
        selected: usize,
        options: Vec<Choice>,
    },
    Input {
        typing: bool,
        input: Vec<char>,
    },
    TimedInfo {
        duration: u64,
        timer: u64,
    },
}

#[derive(Clone)]
pub struct Popup {
    pub title: String,
    pub message: String,
    pub variant: Popups,
}

#[derive(Clone)]
pub struct Choice {
    pub display_text: String,
}

#[derive(Clone)]
pub enum Output {
    Empty,
    String(String),
    Index(usize),
}

#[repr(usize)]
#[derive(PartialEq)]
pub enum BoolChoice {
    False = 0,
    True = 1,
}

impl From<usize> for BoolChoice {
    fn from(input: usize) -> Self {
        match input {
            0 => Self::False,
            1 => Self::True,
            _ => {
                eprintln!("Provided Bool Choice does not exist");
                Self::False
            }
        }
    }
}

impl BoolChoice {
    #[must_use]
    pub fn is_true(&self) -> bool {
        *self == Self::True
    }

    #[must_use]
    pub const fn yes_no_display(&self) -> &str {
        match *self {
            Self::False => "No",
            Self::True => "Yes",
        }
    }
}

#[must_use]
fn get_bool_choices() -> Vec<Choice> {
    vec![
        Choice {
            display_text: String::from(BoolChoice::False.yes_no_display()),
        },
        Choice {
            display_text: String::from(BoolChoice::True.yes_no_display()),
        },
    ]
}

#[must_use]
pub fn get_chat_choice() -> Popup {
    Popup {
        title: String::from("Launch Chat"),
        message: String::from("Do you want to launch the chat with the stream?"),
        variant: Popups::Choice {
            selected: 0,
            options: get_bool_choices(),
        },
    }
}
