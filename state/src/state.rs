use crate::{
    app_state::{
        exit::Exit,
        home::Home,
        popup::Popup,
        startup::{AccountMissing, Startup},
    },
    event::Event,
    transition::Transition,
};
use async_trait::async_trait;
use crossterm::event::KeyEvent;
use std::fmt::{Display, Formatter, Result};
use tokio::sync::mpsc::UnboundedSender;
use tui::{backend::Backend, terminal::Frame};
use twitch::account::Account;
use ui::theme::Theme;

#[async_trait]
pub trait State {
    async fn tick(&self, account: &Option<Account>, timer: u64, events: UnboundedSender<Event>);

    fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>, timer: u64);

    fn transition(
        &self,
        event: Event,
        account: &Option<Account>,
        tx: UnboundedSender<Event>,
    ) -> Option<Transition>;

    fn handle(&self, key_event: KeyEvent) -> Option<Event>;

    fn process(&mut self, action: Event, tx: &UnboundedSender<Event>);
}

#[allow(clippy::module_name_repetitions)]
pub enum AppState {
    AccountMissing(AccountMissing),
    Startup(Startup),
    Home(Home),
    Popup(Popup),
    // Follows(StateFollows),
    Exit(Exit),
}

impl AppState {
    pub async fn tick(&self, account: &Option<Account>, timer: u64, tx: UnboundedSender<Event>) {
        match self {
            Self::AccountMissing(state) => state.tick(account, timer, tx).await,
            Self::Startup(state) => state.tick(account, timer, tx).await,
            Self::Home(state) => state.tick(account, timer, tx).await,
            Self::Popup(state) => state.tick(account, timer, tx).await,
            Self::Exit(state) => state.tick(account, timer, tx).await,
        }
    }

    pub fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>, timer: u64) {
        match self {
            Self::AccountMissing(state) => state.render(theme, frame, timer),
            Self::Startup(state) => state.render(theme, frame, timer),
            Self::Home(state) => state.render(theme, frame, timer),
            Self::Popup(state) => state.render(theme, frame, timer),
            Self::Exit(state) => state.render(theme, frame, timer),
        }
    }

    #[must_use]
    pub fn transition(
        &self,
        account: &Option<Account>,
        event: Event,
        events_sender: UnboundedSender<Event>,
    ) -> Option<Transition> {
        match self {
            Self::AccountMissing(state) => state.transition(event, account, events_sender),
            Self::Startup(state) => state.transition(event, account, events_sender),
            Self::Home(state) => state.transition(event, account, events_sender),
            Self::Popup(state) => state.transition(event, account, events_sender),
            Self::Exit(state) => state.transition(event, account, events_sender),
        }
    }

    pub fn handle(&mut self, key_event: KeyEvent) -> Option<Event> {
        match self {
            Self::AccountMissing(state) => state.handle(key_event),
            Self::Startup(state) => state.handle(key_event),
            Self::Home(state) => state.handle(key_event),
            Self::Popup(state) => state.handle(key_event),
            Self::Exit(state) => state.handle(key_event),
        }
    }

    pub fn receive(&mut self) {
        match self {
            Self::Home(state) => state.channel_check(),
            Self::AccountMissing(_) | Self::Startup(_) | Self::Popup(_) | Self::Exit(_) => {}
        }
    }

    pub fn process(&mut self, action: Event, tx: &UnboundedSender<Event>) {
        match self {
            Self::AccountMissing(state) => state.process(action, tx),
            Self::Startup(state) => state.process(action, tx),
            Self::Home(state) => state.process(action, tx),
            Self::Popup(state) => state.process(action, tx),
            Self::Exit(state) => state.process(action, tx),
        }
    }
}

// TODO state enter and exit?

// TODO move to utils
// #[allow(clippy::integer_arithmetic, clippy::integer_division)]
// const fn ticks_from_seconds(tick_rate: u64, seconds: u64) -> u64 {
//     (1000_u64 / tick_rate) * seconds
// }

#[derive(Clone)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

impl Display for MoveDirection {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Self::Up => write!(f, "Up"),
            Self::Down => write!(f, "Down"),
            Self::Left => write!(f, "Left"),
            Self::Right => write!(f, "Right"),
        }
    }
}

#[derive(Clone)]
pub enum MoveEnd {
    First,
    Last,
}

impl Display for MoveEnd {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Self::First => write!(f, "First"),
            Self::Last => write!(f, "Last"),
        }
    }
}
