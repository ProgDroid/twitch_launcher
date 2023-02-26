use crate::{
    app_state::{
        exit::Exit,
        home::Home,
        popup::{
            client_id_submit, client_secret_submit, redirect_url_port_submit, user_id_submit,
            username_submit, Callback, Popup,
        },
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
                let channels = Channel::load_from_file(CHANNELS_FILE)
                    .map_or_else(|_| Vec::new(), |channels| channels);

                Some(Transition::To(AppState::Home(Home::init(
                    channels.as_slice(),
                    &tx,
                ))))
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
    pub account_config: AccountMissingConfig,
    pub input_handler: Handler<Event>,
}

#[derive(Clone)]
pub struct AccountMissingConfig {
    callback: Option<Callback>,
    title: Option<String>,
    username: Option<String>,
    user_id: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    port: Option<u16>,
}

impl Default for AccountMissingConfig {
    fn default() -> Self {
        Self {
            callback: Some(username_submit),
            title: Some(String::from("Username")),
            username: None,
            user_id: None,
            client_id: None,
            client_secret: None,
            port: None,
        }
    }
}

impl AccountMissing {
    pub fn new(timer: u64, duration: u64, account_config: AccountMissingConfig) -> Self {
        Self {
            timer,
            duration,
            account_config,
            input_handler: Handler::new(Vec::new()),
        }
    }
}

impl Default for AccountMissing {
    fn default() -> Self {
        Self::new(0, STARTUP_DURATION, AccountMissingConfig::default())
    }
}

#[async_trait]
impl State for AccountMissing {
    async fn tick(&self, _: &Option<Account>, timer: u64, events: UnboundedSender<Event>) {
        if timer > self.duration {
            if let Some(callback) = self.account_config.callback {
                if let Some(title) = &self.account_config.title {
                    let _result = events.send(Event::InputPopupStarted((
                        String::from(title),
                        format!("Your {title} here"),
                        Some(callback),
                    )));

                    return;
                }
            }

            #[allow(clippy::expect_used)]
            match Account::new(
                self.account_config
                    .username
                    .clone()
                    .expect("Missing username correctly after setting port"),
                self.account_config
                    .user_id
                    .clone()
                    .expect("Missing user ID correctly after setting port"),
                self.account_config
                    .client_id
                    .clone()
                    .expect("Missing client ID correctly after setting port"),
                self.account_config
                    .client_secret
                    .clone()
                    .expect("Missing client secret correctly after setting port"),
                self.account_config
                    .port
                    .expect("Missing port after setting it"),
            )
            .await
            {
                Ok(account) => {
                    let _result = events.send(Event::AccountConfigured(account));
                }
                Err(e) => {
                    eprintln!("Could not set new account: {e}");
                    let _result = events.send(Event::Exited);
                }
            }
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
                let channels = Channel::load_from_file(CHANNELS_FILE)
                    .map_or_else(|_| Vec::new(), |channels| channels);

                Some(Transition::To(AppState::Home(Home::init(
                    channels.as_slice(),
                    &tx,
                ))))
            }
            Event::InputPopupStarted((title, message, callback)) => Some(Transition::Push(
                AppState::Popup(Popup::new_input(title, message, callback)),
            )),
            Event::SetUser(username) => {
                let mut config = self.account_config.clone();

                config.callback = Some(user_id_submit);
                config.title = Some(String::from("User ID"));
                config.username = Some(username);

                Some(new_account_missing(config))
            }
            Event::SetUserId(user_id) => {
                let mut config = self.account_config.clone();

                config.callback = Some(client_id_submit);
                config.title = Some(String::from("Client ID"));
                config.user_id = Some(user_id);

                Some(new_account_missing(config))
            }
            Event::SetClientId(client_id) => {
                let mut config = self.account_config.clone();

                config.callback = Some(client_secret_submit);
                config.title = Some(String::from("Client Secret"));
                config.client_id = Some(client_id);

                Some(new_account_missing(config))
            }
            Event::SetClientSecret(client_secret) => {
                let mut config = self.account_config.clone();

                config.callback = Some(redirect_url_port_submit);
                config.title = Some(String::from("Redirect URL Port"));
                config.client_secret = Some(client_secret);

                Some(new_account_missing(config))
            }
            Event::SetRedirectUrlPort(port) => {
                let mut config = self.account_config.clone();

                config.callback = None;
                config.title = None;
                config.port = Some(port);

                Some(new_account_missing(config))
            }
            _ => None,
        }
    }

    fn handle(&self, key_event: KeyEvent) -> Option<Event> {
        self.input_handler.handle(key_event)
    }

    fn process(&mut self, _: Event, _: &UnboundedSender<Event>) {}
}

fn new_account_missing(account_config: AccountMissingConfig) -> Transition {
    Transition::To(AppState::AccountMissing(AccountMissing::new(
        0,
        2,
        account_config,
    )))
}
