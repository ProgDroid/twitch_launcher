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
