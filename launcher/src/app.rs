use crate::input_mapping::app_inputs;
use app_event::event::Event;
use crossterm::event::KeyEvent;
use input::handler::Handler;
use state::state_machine::StateMachine;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
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
    paste_sender: UnboundedSender<String>,
}

impl App {
    pub async fn new() -> Self {
        let account: Option<Account> = (Account::load().await).ok();

        let (sender, receiver) = mpsc::unbounded_channel();

        let (paste_sender, paste_receiver) = mpsc::unbounded_channel();

        // TODO load inputs

        Self {
            running: true,
            theme: Theme::default(),
            state: StateMachine::new(account.is_some(), sender, paste_receiver),
            events: receiver,
            input_handler: Handler::new(app_inputs()),
            account,
            paste_sender,
        }
    }

    pub async fn tick(&mut self) {
        self.state.tick(&self.account).await;

        if let Ok(event) = self.events.try_recv() {
            match event {
                Event::Exit => self.running = false,
                Event::SetTheme(theme) => self.theme = theme,
                Event::SetAccount(account) => self.account = Some(account),
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
                Event::SetTheme(_) | Event::SetAccount(_) => {}
            }
        }

        self.state.handle(key_event);
    }

    pub fn handle_paste(&mut self, content: String) {
        if content.trim().is_empty() {
            return;
        }

        let _result = self.paste_sender.send(content);
    }

    pub fn receive(&mut self) {
        self.state.receive();
    }

    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.running
    }
}
