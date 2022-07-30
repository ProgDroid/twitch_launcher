use crate::state::{MoveDirection, MoveEnd};
use std::fmt::{Display, Formatter, Result};
use twitch::channel::Channel;

// TODO split this into action and event? for keybinds. something like `CycleTab` would be ambiguous

#[derive(Clone)]
pub enum Event {
    Started,
    Exited,
    CheckChannels(Vec<Channel>),
    ChannelSelected(Channel, bool),
    ChoicePopupStarted((String, String, Vec<String>)),
    InputPopupStarted((String, String)),
    TimedInfoPopupStarted((String, String, u64)),
    PopupEnded,
    // PopupOutput(Output),
    ChatChoice(bool),
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
            Self::ChannelSelected(channel, _) => write!(f, "Channel {} selected", channel.handle),
            Self::ChoicePopupStarted(_) => write!(f, "Choice Popup started"),
            Self::InputPopupStarted(_) => write!(f, "Input Popup started"),
            Self::TimedInfoPopupStarted(_) => write!(f, "Timed Info Popup started"),
            Self::PopupEnded => write!(f, "Popup End"),
            // Self::PopupOutput(output) => write!(f, "Popup Output {}", output),
            Self::ChatChoice(choice) => write!(f, "Chat Choice: {}", choice),
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
