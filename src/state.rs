use crate::{
    channel::{load_channels, load_channels_from_file, Channel, Status},
    handler::{keybinds_exit, keybinds_home, keybinds_popup, keybinds_startup, keybinds_typing},
    keybind::Keybind,
    panel::Home,
    popup::{BoolChoice, Output, Popup, Popups},
    render,
    secret::{Expose, Secret},
    theme::Theme,
    twitch_account::TwitchAccount,
};
use async_trait::async_trait;
use std::{collections::VecDeque, mem::discriminant};
use tokio::sync::mpsc::{self, error::TryRecvError};
use tui::{backend::Backend, terminal::Frame};

pub type TabTitles = [&'static str; 1];

// TODO should this be enum?
const TAB_TITLES: TabTitles = ["Home"];

pub enum Transition {
    Push(StateMachine),
    Pop,
    To(StateMachine),
}

pub type EventCallback = fn(&Output) -> Event;

#[must_use]
#[allow(clippy::pattern_type_mismatch)]
pub fn chat_choice(output: &Output) -> Event {
    if let Output::Index(choice) = output {
        return Event::ChatChoice(BoolChoice::from(*choice));
    }

    Event::ChatChoice(BoolChoice::False)
}

pub enum Event {
    AccountLoaded,
    Exited,
    ChannelSelected(Channel, bool),
    PopupStart(Popup),
    PopupEnded(Output),
    ChatChoice(BoolChoice),
}

#[derive(Default)]
pub struct Cache {
    storage: Vec<StateMachine>,
}

impl Cache {
    #[allow(clippy::integer_arithmetic)]
    pub fn add(&mut self, state: &mut StateMachine) -> usize {
        if let Some(index) = (&self.storage)
            .iter()
            .position(|cached_state| discriminant(state) == discriminant(cached_state))
        {
            if let Some(existing_state) = self.storage.get_mut(index) {
                *existing_state = (*state).clone();
                return index;
            }

            eprintln!("State found in cache but could not be grabbed");
        }

        self.storage.push((*state).clone());
        self.storage.len() - 1
    }

    pub fn get(&mut self, index: usize) -> Option<StateMachine> {
        if let Some(state) = self.storage.get_mut(index) {
            return Some((*state).clone());
        }

        None
    }
}

#[allow(clippy::module_name_repetitions)]
pub enum StateMachine {
    Startup {
        account_loaded: bool,
        timer: u64,
        startup_duration: u64,
        twitch_account: Option<TwitchAccount>,
        event_output: Output,
        event_callback: Option<EventCallback>,
    },
    Home {
        tab: usize,
        tab_titles: TabTitles,
        channel_highlight: usize,
        channels: Vec<Channel>,
        twitch_account: Option<TwitchAccount>,
        channel_check: Option<mpsc::Receiver<(String, Status)>>,
        typing: bool,
        search_input: Vec<char>,
        focused_panel: Home,
        event_output: Output,
        event_callback: Option<EventCallback>,
    },
    Popup {
        popup: Popup,
    },
    // Follows(StateFollows),
    Exit,
}

#[async_trait]
pub trait State {
    fn keybinds(&self) -> Vec<Keybind>;

    async fn tick(&mut self, events: &mut VecDeque<Event>) -> bool;

    fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>);

    fn transition(&self, event: Event) -> Option<Transition>;

    fn clone(&mut self) -> StateMachine;

    fn get_event_output(&self) -> &Output;

    fn set_event_output(&mut self, output: Output);
}

#[async_trait]
impl State for StateMachine {
    fn keybinds(&self) -> Vec<Keybind> {
        match *self {
            Self::Startup { .. } => keybinds_startup(),
            Self::Home { typing, .. } => {
                if typing {
                    keybinds_typing()
                } else {
                    keybinds_home()
                }
            }
            Self::Popup { ref popup, .. } => {
                if let Popups::Input { typing, .. } = popup.variant {
                    if typing {
                        return keybinds_typing();
                    }
                }

                keybinds_popup()
            }
            Self::Exit => keybinds_exit(),
        }
    }

    #[allow(
        clippy::pattern_type_mismatch,
        clippy::integer_arithmetic,
        clippy::indexing_slicing,
        clippy::too_many_lines
    )]
    async fn tick(&mut self, events: &mut VecDeque<Event>) -> bool {
        match self {
            Self::Startup {
                account_loaded,
                ref mut timer,
                startup_duration,
                ref mut event_output,
                ref mut event_callback,
                ..
            } => {
                *timer += 1;

                if *timer > (*startup_duration) {
                    events.push_back(if *account_loaded {
                        Event::AccountLoaded
                    } else {
                        Event::Exited
                    });
                }

                if let Some(callback) = event_callback {
                    match &event_output {
                        Output::Index(_) | Output::String(_) => {
                            events.push_back((callback)(event_output));
                        }
                        Output::Empty => {}
                    };

                    *event_output = Output::Empty;
                    *event_callback = None;
                }

                true
            }
            Self::Home {
                twitch_account,
                channels,
                ref mut channel_check,
                ref mut event_output,
                ref mut event_callback,
                ..
            } => {
                if let Some(check) = channel_check {
                    while let Ok((handle, status)) = check.try_recv() {
                        #[allow(clippy::expect_used)]
                        let index: usize = channels
                            .iter()
                            .position(|channel| {
                                channel.handle == handle && channel.status != status
                            })
                            .expect("Received channel status for non-existing channel");
                        channels[index].status = status;
                    }

                    if check.try_recv() == Err(TryRecvError::Disconnected) {
                        check.close();
                        *channel_check = None;
                    }
                } else {
                    let mut channels_awaiting: Vec<Channel> = Vec::new();

                    for channel in channels {
                        if channel.status == Status::Awaiting {
                            channels_awaiting.push(channel.clone());
                        }
                    }

                    if !channels_awaiting.is_empty() {
                        #[allow(clippy::expect_used)]
                        if let Ok((_, check)) = load_channels(
                            twitch_account
                                .as_ref()
                                .expect("Made it past startup without loading Twitch account"),
                            &channels_awaiting,
                        ) {
                            *channel_check = Some(check);
                        }
                    }
                }

                if let Some(callback) = event_callback {
                    match event_output {
                        Output::Index(_) | Output::String(_) => {
                            events.push_back((callback)(event_output));

                            *event_output = Output::Empty;
                            *event_callback = None;
                        }
                        Output::Empty => {}
                    }
                }

                true
            }
            Self::Popup { popup, .. } => {
                match popup.variant {
                    Popups::TimedInfo {
                        duration,
                        ref mut timer,
                        ..
                    } => {
                        *timer += 1;

                        if *timer > (duration) {
                            events.push_back(Event::PopupEnded(Output::Empty));
                        }
                    }
                    Popups::Input { .. } | Popups::Choice { .. } => {}
                }

                true
            }
            Self::Exit => false,
        }
    }

    fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>) {
        match *self {
            Self::Startup {
                account_loaded,
                timer,
                ..
            } => {
                if account_loaded {
                    render::startup_animation(theme, frame, &timer);
                } else {
                    render::account_missing(theme, frame, &timer);
                }
            }
            Self::Home {
                tab,
                tab_titles,
                channel_highlight,
                ref channels,
                typing,
                ref search_input,
                ref focused_panel,
                ..
            } => render::home(
                theme,
                frame,
                &tab,
                &tab_titles,
                &channel_highlight,
                channels,
                &typing,
                search_input,
                focused_panel,
                &self.keybinds(),
            ),
            Self::Popup { ref popup, .. } => render::popup(theme, frame, popup, &self.keybinds()),
            Self::Exit { .. } => {}
        }
    }

    #[allow(
        clippy::unwrap_in_result,
        clippy::pattern_type_mismatch,
        clippy::too_many_lines
    )]
    fn transition(&self, event: Event) -> Option<Transition> {
        match (self, event) {
            (StateMachine::Startup { twitch_account, .. }, Event::AccountLoaded) => {
                #[allow(clippy::expect_used)]
                let account = twitch_account
                    .as_ref()
                    .expect("Made it past startup without loading Twitch account");

                let (channels, channel_check) = match load_channels_from_file(account) {
                    Ok((channels, channel_check)) => (channels, channel_check),
                    Err(e) => {
                        eprintln!("Error loading channels: {}", e);
                        return Some(Transition::To(Self::Exit));
                    }
                };

                Some(Transition::To(Self::Home {
                    tab: 0,
                    tab_titles: TAB_TITLES,
                    channel_highlight: 0,
                    channels,
                    twitch_account: Some(TwitchAccount {
                        username: String::from(account.username.as_str()),
                        user_id: String::from(account.user_id.as_str()),
                        client_id: Secret::new(account.client_id.expose_value().to_owned()),
                        client_secret: Secret::new(account.client_secret.expose_value().to_owned()),
                        user_access_token: Secret::new(
                            account.user_access_token.expose_value().to_owned(),
                        ),
                        refresh_token: Secret::new(account.refresh_token.expose_value().to_owned()),
                    }),
                    channel_check: Some(channel_check),
                    typing: false,
                    search_input: Vec::new(),
                    focused_panel: Home::default(),
                    event_output: Output::Empty,
                    event_callback: None,
                }))
            }
            (Self::Home { .. }, Event::ChannelSelected(channel, chat)) => {
                if let Err(e) = channel.launch() {
                    eprintln!("Error opening stream: {}", e);
                }

                if chat {
                    if let Err(e) = channel.launch_chat() {
                        eprintln!("Error opening chat: {}", e);
                    }
                }

                None
            }
            #[allow(clippy::unnested_or_patterns)]
            (Self::Home { .. }, Event::Exited) | (Self::Startup { .. }, Event::Exited) => {
                Some(Transition::To(Self::Exit))
            }
            (Self::Home { .. }, Event::PopupStart(popup)) => {
                Some(Transition::Push(Self::Popup { popup }))
            }
            (Self::Popup { .. }, Event::Exited) => Some(Transition::To(Self::Exit)),
            (Self::Popup { .. }, Event::PopupEnded(_)) => Some(Transition::Pop),
            (
                Self::Home {
                    channels,
                    channel_highlight,
                    search_input,
                    focused_panel,
                    ..
                },
                Event::ChatChoice(choice),
            ) => {
                let channel: Channel = match *focused_panel {
                    Home::Favourites => {
                        if let Some(channel) = channels.get(*channel_highlight) {
                            (*channel).clone()
                        } else {
                            return None;
                        }
                    }
                    Home::Search => {
                        if search_input.is_empty() {
                            return None;
                        }

                        let handle: String = search_input.iter().collect();

                        // TODO allow setting to determine whether to launch on browser or locally
                        Channel {
                            friendly_name: String::new(),
                            handle,
                            status: Status::Unknown,
                        }
                    }
                };

                if let Err(e) = channel.launch() {
                    eprintln!("Error opening stream: {}", e);
                }

                if choice.is_true() {
                    if let Err(e) = channel.launch_chat() {
                        eprintln!("Error opening chat: {}", e);
                    }
                }

                None
            }
            _ => None,
        }
    }

    #[allow(clippy::pattern_type_mismatch)]
    fn clone(&mut self) -> Self {
        match self {
            Self::Startup {
                account_loaded,
                timer,
                startup_duration,
                twitch_account,
                event_output,
                event_callback,
                ..
            } => Self::Startup {
                account_loaded: *account_loaded,
                timer: *timer,
                startup_duration: *startup_duration,
                twitch_account: (*twitch_account).clone(),
                event_output: (*event_output).clone(),
                event_callback: *event_callback,
            },
            Self::Home {
                tab,
                tab_titles,
                channel_highlight,
                channels,
                twitch_account,
                ref mut channel_check,
                typing,
                search_input,
                focused_panel,
                event_output,
                event_callback,
                ..
            } => {
                if let Some(check) = channel_check {
                    check.close();
                }

                Self::Home {
                    tab: *tab,
                    tab_titles: *tab_titles,
                    channel_highlight: *channel_highlight,
                    channels: (*channels).clone(),
                    twitch_account: (*twitch_account).clone(),
                    channel_check: None,
                    typing: *typing,
                    search_input: (*search_input).clone(),
                    focused_panel: *focused_panel,
                    event_output: (*event_output).clone(),
                    event_callback: *event_callback,
                }
            }
            Self::Popup { popup } => Self::Popup {
                popup: (*popup).clone(),
            },
            Self::Exit => Self::Exit,
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn get_event_output(&self) -> &Output {
        match *self {
            Self::Startup {
                ref event_output, ..
            }
            | Self::Home {
                ref event_output, ..
            } => event_output,
            _ => &Output::Empty,
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn set_event_output(&mut self, output: Output) {
        match *self {
            Self::Startup {
                ref mut event_output,
                ..
            }
            | Self::Home {
                ref mut event_output,
                ..
            } => *event_output = output,
            _ => {}
        }
    }
}
