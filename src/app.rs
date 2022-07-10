use crate::{
    state::{Event, State, StateMachine},
    theme::Theme,
    twitch_account::TwitchAccount,
};
use std::{collections::VecDeque, error};
use tui::{backend::Backend, terminal::Frame};

const STARTUP_DURATION: u16 = 2;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct App {
    pub running: bool,
    pub theme: Theme,
    pub tick_rate: u64,
    pub state: StateMachine,
    pub events: VecDeque<Event>,
}

impl App {
    pub async fn new(tick_rate: u64) -> Self {
        let twitch_account: Option<TwitchAccount> = match TwitchAccount::load().await {
            Ok(account) => Some(account),
            Err(_) => None,
        };

        let account_loaded = match twitch_account {
            Some(_) => true,
            None => false,
        };

        Self {
            running: true,
            theme: Theme::default(),
            tick_rate,
            state: StateMachine::Startup {
                account_loaded,
                timer: 0,
                startup_duration: ticks_from_seconds(tick_rate, STARTUP_DURATION as u64) as u16,
                twitch_account,
                popup: None,
            },
            events: VecDeque::new(),
        }
    }

    pub async fn tick(&mut self) {
        self.running = self.state.tick(&mut self.events).await;

        match self.events.pop_front() {
            Some(e) => match self.state.transition(e) {
                Some(s) => {
                    self.state = s;
                }
                None => {}
            },
            None => {}
        };
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        self.state.render(&self.theme, frame);
    }
}

fn ticks_from_seconds(tick_rate: u64, seconds: u64) -> u64 {
    (1000u64 / tick_rate) * seconds
}