use crate::{
    channel::{load_channels, load_channels_from_file, Channel, Status},
    handler::{keybinds_exit, keybinds_home, keybinds_popup, keybinds_startup, keybinds_typing},
    keybind::Keybind,
    panel::Home,
    popup::{Popup, Popups},
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

pub enum Event {
    AccountLoaded,
    Exited,
    ChannelSelected(Channel, bool),
    PopupStart,
    PopupEnded,
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
    },
    Popup {
        popup: Popup,
        option_highlight: usize,
        typing: bool,
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
            Self::Popup { typing, .. } => {
                if typing {
                    keybinds_typing()
                } else {
                    keybinds_popup()
                }
            }
            Self::Exit => keybinds_exit(),
        }
    }

    #[allow(
        clippy::pattern_type_mismatch,
        clippy::integer_arithmetic,
        clippy::indexing_slicing
    )]
    async fn tick(&mut self, events: &mut VecDeque<Event>) -> bool {
        match self {
            Self::Startup {
                account_loaded,
                ref mut timer,
                startup_duration,
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

                true
            }
            Self::Home {
                twitch_account,
                channels,
                ref mut channel_check,
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

                    if channels_awaiting.is_empty() {
                        return true;
                    }

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
                            events.push_back(Event::PopupEnded);
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
            Self::Popup { ref popup, .. } => render::popup(theme, frame, popup),
            Self::Exit { .. } => {}
        }
    }

    #[allow(clippy::unwrap_in_result, clippy::pattern_type_mismatch)]
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
            (Self::Home { .. }, Event::PopupStart) => Some(Transition::Push(Self::Popup {
                popup: Popup {
                    title: String::from("Test"),
                    message: String::from("Testerino"),
                    variant: Popups::TimedInfo {
                        duration: 10,
                        timer: 0,
                    },
                },
                option_highlight: 0,
                typing: false,
            })),
            (Self::Popup { .. }, Event::PopupEnded) => Some(Transition::Pop),
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
            } => Self::Startup {
                account_loaded: *account_loaded,
                timer: *timer,
                startup_duration: *startup_duration,
                twitch_account: (*twitch_account).clone(),
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
                }
            }
            Self::Popup {
                popup,
                option_highlight,
                typing,
            } => Self::Popup {
                popup: (*popup).clone(),
                option_highlight: *option_highlight,
                typing: *typing,
            },
            Self::Exit => Self::Exit,
        }
    }
}
