use crate::{
    channel::{load_channels, Channel, ChannelStatus},
    handler::{keybinds_exit, keybinds_home, keybinds_startup, keybinds_typing},
    keybind::Keybind,
    panel::HomePanel,
    popup::Popup,
    render,
    secret::{ExposeSecret, Secret},
    theme::Theme,
    twitch_account::TwitchAccount,
};
use async_trait::async_trait;
use std::collections::VecDeque;
use tokio::sync::mpsc;
use tui::{backend::Backend, terminal::Frame};

pub type TabTitles = [&'static str; 1];

const TAB_TITLES: TabTitles = ["Home"];

pub enum Event {
    AccountLoaded,
    Exited,
    ChannelSelected(Channel, bool),
}

pub enum StateMachine {
    Startup {
        account_loaded: bool,
        timer: u64,
        startup_duration: u16,
        twitch_account: Option<TwitchAccount>,
        popup: Option<Popup>,
    },
    Home {
        tab: usize,
        tab_titles: TabTitles,
        channel_highlight: usize,
        channels: Vec<Channel>,
        twitch_account: Option<TwitchAccount>,
        channel_check: mpsc::Receiver<(String, ChannelStatus)>,
        popup: Option<Popup>,
        typing: bool,
        search_input: Vec<char>,
        focused_panel: HomePanel,
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
        match self {
            StateMachine::Startup { .. } => keybinds_startup(),
            StateMachine::Home { typing, .. } => {
                if *typing {
                    keybinds_typing()
                } else {
                    keybinds_home()
                }
            }
            StateMachine::Exit => keybinds_exit(),
        }
    }

    async fn tick(&mut self, events: &mut VecDeque<Event>) -> bool {
        match self {
            StateMachine::Startup {
                account_loaded,
                ref mut timer,
                startup_duration,
                ..
            } => {
                *timer += 1;

                if *timer > (*startup_duration).into() {
                    events.push_back(if *account_loaded {
                        Event::AccountLoaded
                    } else {
                        Event::Exited
                    })
                }

                true
            }
            StateMachine::Home {
                channels,
                channel_check,
                ..
            } => {
                match channel_check.try_recv() {
                    Ok((handle, status)) => {
                        let index: usize = channels
                            .iter()
                            .position(|channel| channel.handle == handle)
                            .unwrap();
                        channels[index].status = status;
                    }
                    Err(_) => {}
                }

                true
            }
            StateMachine::Exit => false,
        }
    }

    fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>) {
        match self {
            StateMachine::Startup {
                account_loaded,
                timer,
                ..
            } => {
                if *account_loaded {
                    render::startup_animation(theme, frame, &timer)
                } else {
                    render::account_missing(theme, frame, &timer)
                }
            }
            StateMachine::Home {
                tab,
                tab_titles,
                channel_highlight,
                channels,
                popup,
                typing,
                search_input,
                focused_panel,
                ..
            } => render::render_home(
                theme,
                frame,
                tab,
                tab_titles,
                channel_highlight,
                channels,
                popup,
                typing,
                search_input,
                focused_panel,
                &self.keybinds(),
            ),
            StateMachine::Exit { .. } => {}
        }
    }

    fn transition(&self, event: Event) -> Option<StateMachine> {
        match (self, event) {
            (StateMachine::Startup { twitch_account, .. }, Event::AccountLoaded) => {
                let (channels, channel_check) = match load_channels(
                    twitch_account
                        .as_ref()
                        .expect("Made it past startup without loading Twitch account"),
                ) {
                    Ok((channels, channel_check)) => (channels, channel_check),
                    Err(e) => {
                        eprintln!("Error loading channels: {}", e);
                        return Some(StateMachine::Exit);
                    }
                };

                Some(StateMachine::Home {
                    tab: 0,
                    tab_titles: TAB_TITLES,
                    channel_highlight: 0,
                    channels,
                    twitch_account: Some(TwitchAccount {
                        username: String::from(
                            (twitch_account.as_ref().unwrap()).username.as_str(),
                        ),
                        user_id: String::from((*twitch_account.as_ref().unwrap()).user_id.as_str()),
                        client_id: Secret::new(
                            (twitch_account.as_ref().unwrap())
                                .client_id
                                .expose_value()
                                .to_string(),
                        ),
                        client_secret: Secret::new(
                            (twitch_account.as_ref().unwrap())
                                .client_secret
                                .expose_value()
                                .to_string(),
                        ),
                        user_access_token: Secret::new(
                            (twitch_account.as_ref().unwrap())
                                .user_access_token
                                .expose_value()
                                .to_string(),
                        ),
                        refresh_token: Secret::new(
                            (twitch_account.as_ref().unwrap())
                                .refresh_token
                                .expose_value()
                                .to_string(),
                        ),
                    }),
                    channel_check,
                    popup: None,
                    typing: false,
                    search_input: Vec::new(),
                    focused_panel: HomePanel::default(),
                })
            }
            (StateMachine::Home { .. }, Event::Exited) => Some(StateMachine::Exit),
            (StateMachine::Home { .. }, Event::ChannelSelected(channel, chat)) => {
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
            (StateMachine::Startup { .. }, Event::Exited) => Some(StateMachine::Exit),
            _ => None,
        }
    }
}
