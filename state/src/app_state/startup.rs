use crate::{
    app_state::{
        exit::Exit,
        home::Home,
        popup::{client_id_submit, client_secret_submit, Callback, Popup},
    },
    event::Event,
    state::{AppState, State},
    transition::Transition,
};
use async_trait::async_trait;
use crossterm::event::KeyEvent;
use input::handler::Handler;
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
    pub callback: Option<Callback>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub input_handler: Handler<Event>,
}

impl AccountMissing {
    pub fn new(
        timer: u64,
        duration: u64,
        callback: Option<Callback>,
        client_id: Option<String>,
        client_secret: Option<String>,
    ) -> Self {
        Self {
            timer,
            duration,
            callback,
            client_id,
            client_secret,
            input_handler: Handler::new(Vec::new()),
        }
    }
}

impl Default for AccountMissing {
    fn default() -> Self {
        Self::new(0, STARTUP_DURATION, Some(client_id_submit), None, None)
    }
}

#[async_trait]
impl State for AccountMissing {
    #[allow(clippy::integer_arithmetic, clippy::integer_division)]
    async fn tick(&self, _: &Option<Account>, timer: u64, events: UnboundedSender<Event>) {
        if timer > self.duration {
            if let Some(callback) = self.callback {
                let title: &str = if self.client_id.is_none() {
                    "Client ID"
                } else {
                    "Client Secret"
                };

                let _result = events.send(Event::InputPopupStarted((
                    String::from(title),
                    format!("Your {} here", title),
                    Some(callback),
                )));

                return;
            }

            if let Some(client_id) = &self.client_id {
                if let Some(client_secret) = &self.client_secret {
                    match Account::new(client_id.clone(), client_secret.clone()).await {
                        Ok(account) => {
                            let _result = events.send(Event::AccountConfigured(account));
                        }
                        Err(e) => {
                            eprintln!("Could not set new account: {}", e);
                            let _result = events.send(Event::Exited);
                        }
                    }

                    return;
                }
            }

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
        tx: UnboundedSender<Event>,
    ) -> Option<Transition> {
        match event {
            Event::Exited => Some(Transition::To(AppState::Exit(Exit::new()))),
            Event::AccountConfigured(_) => {
                if let Ok(channels) = Channel::load_from_file(CHANNELS_FILE) {
                    return Some(Transition::To(AppState::Home(Home::init(
                        channels.as_slice(),
                        &tx,
                    ))));
                };

                // Some(Transition::To(AppState::Exit(Exit::new())))
                None
            }
            Event::InputPopupStarted((title, message, callback)) => Some(Transition::Push(
                AppState::Popup(Popup::new_input(title, message, callback)),
            )),
            Event::SetClientId(client_id) => {
                Some(Transition::To(AppState::AccountMissing(Self::new(
                    0,
                    STARTUP_DURATION,
                    Some(client_secret_submit),
                    Some(client_id),
                    None,
                ))))
            }
            Event::SetClientSecret(client_secret) => {
                Some(Transition::To(AppState::AccountMissing(Self::new(
                    0,
                    STARTUP_DURATION,
                    None,
                    self.client_id.clone(),
                    Some(client_secret),
                ))))
            }
            _ => None,
        }
    }

    fn handle(&self, key_event: KeyEvent) -> Option<Event> {
        self.input_handler.handle(key_event)
    }

    fn process(&mut self, _: Event, _: &UnboundedSender<Event>) {}
}
