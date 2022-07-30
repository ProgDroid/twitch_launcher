use crate::{
    app_state::{exit::Exit, home::Home},
    event::Event,
    state::{AppState, State},
    transition::Transition,
};
use async_trait::async_trait;
use crossterm::event::KeyEvent;
use input::{handler::Handler, keybind::KeyBind};
use tokio::sync::mpsc::UnboundedSender;
use tui::{backend::Backend, terminal::Frame};
use twitch::{account::Account, channel::Channel};
use ui::{
    render::startup::{account_missing, starting},
    theme::Theme,
};

const CHANNELS_FILE: &str = "favourites.json";
const STARTUP_DURATION: u64 = 2;

pub struct Startup {
    pub timer: u64,
    pub duration: u64,
    pub input_handler: Handler<Event>,
}

impl Startup {
    pub fn new(timer: u64, duration: u64) -> Self {
        Self {
            timer,
            duration,
            input_handler: Handler::new(Vec::new()),
        }
    }
}

impl Default for Startup {
    fn default() -> Self {
        Self::new(0, STARTUP_DURATION)
    }
}

#[async_trait]
impl State for Startup {
    fn keybinds(&self) -> Vec<KeyBind<Event>> {
        self.input_handler.inputs.clone()
    }

    async fn tick(&self, _: &Option<Account>, timer: u64, events: UnboundedSender<Event>) {
        if timer > self.duration {
            let _result = events.send(Event::Started);
        }
    }

    fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>, timer: u64) {
        starting(theme, frame, timer);
    }

    fn transition(
        &self,
        event: Event,
        _: &Option<Account>,
        tx: UnboundedSender<Event>,
    ) -> Option<Transition> {
        match event {
            Event::Started => {
                if let Ok(channels) = Channel::load_from_file(CHANNELS_FILE) {
                    return Some(Transition::To(AppState::Home(Home::init(
                        channels.as_slice(),
                        &tx,
                    ))));
                };

                Some(Transition::To(AppState::Exit(Exit::new())))
            }
            Event::Exited => Some(Transition::To(AppState::Exit(Exit::new()))),
            _ => None,
        }
    }

    fn handle(&self, key_event: KeyEvent) -> Option<Event> {
        self.input_handler.handle(key_event)
    }

    fn process(&mut self, _: Event, _: &UnboundedSender<Event>) {}
}

pub struct AccountMissing {
    pub timer: u64,
    pub duration: u64,
    pub input_handler: Handler<Event>,
}

impl AccountMissing {
    pub fn new(timer: u64, duration: u64) -> Self {
        Self {
            timer,
            duration,
            input_handler: Handler::new(Vec::new()),
        }
    }
}

impl Default for AccountMissing {
    fn default() -> Self {
        Self::new(0, STARTUP_DURATION)
    }
}

#[async_trait]
impl State for AccountMissing {
    fn keybinds(&self) -> Vec<KeyBind<Event>> {
        self.input_handler.inputs.clone()
    }

    async fn tick(&self, _: &Option<Account>, timer: u64, events: UnboundedSender<Event>) {
        if timer > self.duration {
            let _result = events.send(Event::Exited);
        }
    }

    fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>, timer: u64) {
        account_missing(theme, frame, timer);
    }

    fn transition(
        &self,
        event: Event,
        _: &Option<Account>,
        _: UnboundedSender<Event>,
    ) -> Option<Transition> {
        match event {
            Event::Exited => Some(Transition::To(AppState::Exit(Exit::new()))),
            _ => None,
        }
    }

    fn handle(&self, key_event: KeyEvent) -> Option<Event> {
        self.input_handler.handle(key_event)
    }

    fn process(&mut self, _: Event, _: &UnboundedSender<Event>) {}
}
