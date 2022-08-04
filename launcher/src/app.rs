use crate::input_mapping::app_inputs;
use app_event::event::Event;
use crossterm::event::KeyEvent;
use input::handler::Handler;
use state::state_machine::StateMachine;
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tui::{backend::Backend, terminal::Frame};
use twitch::account::Account;
use ui::theme::Theme;

pub struct App {
    running: bool,
    theme: Theme,
    state: StateMachine,
    events: UnboundedReceiver<Event>,
    input_handler: Handler<Event>,
    account: Option<Account>,
}

impl App {
    pub async fn new() -> Self {
        let account: Option<Account> = match Account::load().await {
            Ok(account) => Some(account),
            Err(_) => None,
        };

        let (sender, receiver) = mpsc::unbounded_channel();

        // TODO load inputs

        Self {
            running: true,
            theme: Theme::default(),
            state: StateMachine::new(account.is_some(), sender),
            events: receiver,
            input_handler: Handler::new(app_inputs()),
            account,
        }
    }

    pub async fn tick(&mut self) {
        self.state.tick(&self.account).await;

        if let Ok(event) = self.events.try_recv() {
            match event {
                Event::Exit => self.running = false,
                Event::SetTheme(theme) => self.theme = theme,
            }
        }
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        self.state.render(&self.theme, frame);
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) {
        if let Some(event) = self.input_handler.handle(key_event) {
            match event {
                Event::Exit => self.running = false,
                Event::SetTheme(_) => {}
            }
        }

        self.state.handle(key_event);
    }

    pub fn receive(&mut self) {
        self.state.receive();
    }

    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.running
    }
}
