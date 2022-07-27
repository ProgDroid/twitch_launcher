use std::fmt::{Display, Formatter, Result};
use twitch::channel::Channel;

#[derive(Clone)]
pub enum Event {
    Started,
    Exited,
    CheckChannels(Vec<Channel>),
    FavouritesLoaded(Vec<Channel>),
    ChannelSelected(Channel, bool),
    ChoicePopupStarted((String, String, Vec<String>)),
    InputPopupStarted((String, String)),
    TimedInfoPopupStarted((String, String, u64)),
    PopupEnded,
    // PopupOutput(Output),
    ChatChoice(bool),
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Started => write!(f, "Started"),
            Self::Exited => write!(f, "Exit"),
            Self::CheckChannels(_) => write!(f, "Check Channels"),
            Self::FavouritesLoaded(_) => write!(f, "Favourite Channels loaded"),
            Self::ChannelSelected(channel, _) => write!(f, "Channel {} selected", channel.handle),
            Self::ChoicePopupStarted(_) => write!(f, "Choice Popup started"),
            Self::InputPopupStarted(_) => write!(f, "Input Popup started"),
            Self::TimedInfoPopupStarted(_) => write!(f, "Timed Info Popup started"),
            Self::PopupEnded => write!(f, "Popup End"),
            // Self::PopupOutput(output) => write!(f, "Popup Output {}", output),
            Self::ChatChoice(choice) => write!(f, "Chat Choice: {}", choice),
        }
    }
}
