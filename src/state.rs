use crate::{
    channel::{load_channels, Channel, ChannelStatus},
    handler::{keybinds_exit, keybinds_home, keybinds_startup, KeyBindFn},
    popup::Popup,
    render,
    secret::{ExposeSecret, Secret},
    theme::Theme,
    twitch_account::TwitchAccount,
};
use async_trait::async_trait;
use crossterm::event::KeyEvent;
use std::collections::VecDeque;
use tokio::sync::mpsc;
use tui::{backend::Backend, terminal::Frame};

pub enum Event {
    AccountLoaded,
    Exited,
    ChannelSelected,
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
        tab_titles: [&'static str; 2],
        channel_highlight: usize,
        channels: Vec<Channel>,
        twitch_account: Option<TwitchAccount>,
        channel_check: mpsc::Receiver<(String, ChannelStatus)>,
        popup: Option<Popup>,
        search_input: String,
    },
    // Follows(StateFollows),
    Exit,
}

#[async_trait]
pub trait State {
    fn keybinds(&self, key_event: KeyEvent) -> Option<KeyBindFn>;

    async fn tick(&mut self, events: &mut VecDeque<Event>) -> bool;

    fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>);

    fn transition(&self, event: Event) -> Option<StateMachine>;
}

#[async_trait]
impl State for StateMachine {
    fn keybinds(&self, key_event: KeyEvent) -> Option<KeyBindFn> {
        match self {
            StateMachine::Startup { .. } => keybinds_startup(key_event),
            StateMachine::Home { .. } => keybinds_home(key_event),
            StateMachine::Exit => keybinds_exit(key_event),
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
                search_input,
                ..
            } => render::render_home(
                theme,
                frame,
                tab,
                tab_titles,
                channel_highlight,
                channels,
                popup,
                search_input,
            ),
            StateMachine::Exit { .. } => {}
        }
    }

    fn transition(&self, event: Event) -> Option<StateMachine> {
        match (self, event) {
            (StateMachine::Startup { twitch_account, .. }, Event::AccountLoaded) => {
                let (channels, channel_check) = load_channels(twitch_account.as_ref().unwrap());

                Some(StateMachine::Home {
                    tab: 0,
                    tab_titles: ["Home", "Follows"], // TODO make default const?
                    channel_highlight: 0,
                    channels,
                    twitch_account: Some(TwitchAccount {
                        username: String::from(
                            (*twitch_account.as_ref().unwrap()).username.as_str(),
                        ),
                        user_id: String::from((*twitch_account.as_ref().unwrap()).user_id.as_str()),
                        client_id: Secret::new(
                            (*twitch_account.as_ref().unwrap())
                                .client_id
                                .expose_value()
                                .to_string(),
                        ),
                        client_secret: Secret::new(
                            (*twitch_account.as_ref().unwrap())
                                .client_secret
                                .expose_value()
                                .to_string(),
                        ),
                        user_access_token: Secret::new(
                            (*twitch_account.as_ref().unwrap())
                                .user_access_token
                                .expose_value()
                                .to_string(),
                        ),
                        refresh_token: Secret::new(
                            (*twitch_account.as_ref().unwrap())
                                .refresh_token
                                .expose_value()
                                .to_string(),
                        ),
                    }),
                    channel_check,
                    popup: None,
                    search_input: String::new(), // TODO should this be Vec<Char>?
                })
            }
            (StateMachine::Home { .. }, Event::Exited) => Some(StateMachine::Exit),
            (
                StateMachine::Home {
                    channel_highlight,
                    channels,
                    ..
                },
                Event::ChannelSelected,
            ) => {
                match channels[*channel_highlight].launch() {
                    (Ok(_), Ok(_)) => {}
                    (Ok(_), Err(e)) => {
                        eprintln!("Error opening chat: {}", e);
                    }
                    (Err(e), Ok(_)) => {
                        eprintln!("Error opening stream: {}", e);
                    }
                    (Err(e_stream), Err(e_chat)) => {
                        eprintln!(
                            "Error opening stream: {}\nError opening chat: {}",
                            e_stream, e_chat
                        );
                    }
                }

                Some(StateMachine::Exit)
            }
            (StateMachine::Startup { .. }, Event::Exited) => Some(StateMachine::Exit),
            _ => None,
        }
    }
}
