use crate::{
    channel::{load_channels, Channel, Status},
    handler::{keybinds_exit, keybinds_home, keybinds_startup, keybinds_typing},
    keybind::Keybind,
    panel::Home,
    popup::Popup,
    render,
    secret::{Expose, Secret},
    theme::Theme,
    twitch_account::TwitchAccount,
};
use async_trait::async_trait;
use std::collections::VecDeque;
use tokio::sync::mpsc;
use tui::{backend::Backend, terminal::Frame};

pub type TabTitles = [&'static str; 1];

// TODO should this be enum?
const TAB_TITLES: TabTitles = ["Home"];

pub enum Event {
    AccountLoaded,
    Exited,
    ChannelSelected(Channel, bool),
}

#[allow(clippy::module_name_repetitions)]
pub enum StateMachine {
    Startup {
        account_loaded: bool,
        timer: u64,
        startup_duration: u64,
        twitch_account: Option<TwitchAccount>,
        popup: Option<Popup>,
    },
    Home {
        tab: usize,
        tab_titles: TabTitles,
        channel_highlight: usize,
        channels: Vec<Channel>,
        twitch_account: Option<TwitchAccount>,
        channel_check: mpsc::Receiver<(String, Status)>,
        popup: Option<Popup>,
        typing: bool,
        search_input: Vec<char>,
        focused_panel: Home,
    },
    // Follows(StateFollows),
    Exit,
}

#[async_trait]
pub trait State {
    fn keybinds(&self) -> Vec<Keybind>;

    async fn tick(&mut self, events: &mut VecDeque<Event>) -> bool;

    fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>);

    fn transition(&self, event: Event) -> Option<StateMachine>;
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
                channels,
                channel_check,
                ..
            } => {
                while let Ok((handle, status)) = channel_check.try_recv() {
                    #[allow(clippy::expect_used)]
                    let index: usize = channels
                        .iter()
                        .position(|channel| channel.handle == handle && channel.status != status)
                        .expect("Received channel status for non-existing channel");
                    channels[index].status = status;
                }

                true
            }
            Self::Exit => false,
        }
    }

    fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>) {
        match *self {
            StateMachine::Startup {
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
            StateMachine::Home {
                tab,
                tab_titles,
                channel_highlight,
                ref channels,
                ref popup,
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
                popup,
                &typing,
                search_input,
                focused_panel,
                &self.keybinds(),
            ),
            StateMachine::Exit { .. } => {}
        }
    }

    #[allow(clippy::unwrap_in_result, clippy::pattern_type_mismatch)]
    fn transition(&self, event: Event) -> Option<StateMachine> {
        match (self, event) {
            (StateMachine::Startup { twitch_account, .. }, Event::AccountLoaded) => {
                #[allow(clippy::expect_used)]
                let account = twitch_account
                    .as_ref()
                    .expect("Made it past startup without loading Twitch account");

                let (channels, channel_check) = match load_channels(account) {
                    Ok((channels, channel_check)) => (channels, channel_check),
                    Err(e) => {
                        eprintln!("Error loading channels: {}", e);
                        return Some(Self::Exit);
                    }
                };

                Some(Self::Home {
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
                    channel_check,
                    popup: None,
                    typing: false,
                    search_input: Vec::new(),
                    focused_panel: Home::default(),
                })
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
                Some(Self::Exit)
            }
            _ => None,
        }
    }
}
