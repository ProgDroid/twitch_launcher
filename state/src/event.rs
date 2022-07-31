use crate::{
    app_state::popup::Callback,
    state::{MoveDirection, MoveEnd},
};
use std::fmt::{Display, Formatter, Result};
use twitch::channel::Channel;

// TODO split this into action and event? for keybinds. something like `CycleTab` would be ambiguous

#[derive(Clone)]
pub enum Event {
    Started,
    Exited,
    CheckChannels(Vec<Channel>),
    ChannelSelected(Channel, bool),
    ChoicePopupStarted((String, String, Vec<String>, Option<Callback>)),
    InputPopupStarted((String, String, Option<Callback>)),
    TimedInfoPopupStarted((String, String, u64, Option<Callback>)),
    PopupEnded,
    ChatChoice(usize),
    ChatChoiceSearch(usize),
    CycleTab(MoveDirection),
    CycleHighlight(MoveDirection),
    HomeEndHighlight(MoveEnd),
    Selected,
    CyclePanel(MoveDirection),
    StopTyping,
    Submit,
    DeleteChar,
    Typed(char),
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Started => write!(f, "Started"),
            Self::Exited => write!(f, "Exit"),
            Self::CheckChannels(_) => write!(f, "Check Channels"),
            Self::ChannelSelected(channel, choice) => write!(
                f,
                "Channel {} selected {} chat",
                channel.handle,
                if *choice { "with" } else { "without" }
            ),
            Self::ChoicePopupStarted(_) => write!(f, "Choice Popup started"),
            Self::InputPopupStarted(_) => write!(f, "Input Popup started"),
            Self::TimedInfoPopupStarted(_) => write!(f, "Timed Info Popup started"),
            Self::PopupEnded => write!(f, "Popup End"),
            Self::ChatChoice(choice) => write!(f, "Chat Choice: {}", choice),
            Self::ChatChoiceSearch(choice) => write!(f, "Chat Choice from Search: {}", choice),
            Self::CycleTab(direction) => write!(f, "Cycle Tab {}", direction),
            Self::CycleHighlight(direction) => write!(f, "Cycle Highlight {}", direction),
            Self::HomeEndHighlight(end) => write!(f, "Highlight to {}", end),
            Self::Selected => write!(f, "Selected current highlight"),
            Self::CyclePanel(direction) => write!(f, "Cycle Panel {}", direction),
            Self::StopTyping => write!(f, "Stop Typing"),
            Self::Submit => write!(f, "Submit"),
            Self::DeleteChar => write!(f, "Delete Char"),
            Self::Typed(char) => write!(f, "Typed {}", char),
        }
    }
}
