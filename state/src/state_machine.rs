use crate::{
    app_state::startup::{AccountMissing, Startup},
    cache::Cache,
    event::Event,
    state::AppState,
    transition::Transition,
};
use app_event::event::Event as AppEvent;
use crossterm::event::KeyEvent;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tui::{backend::Backend, terminal::Frame};
use twitch::account::Account;
use ui::theme::Theme;

pub struct StateMachine {
    state: AppState,
    cache: Cache,
    stack: Vec<usize>,
    events: UnboundedReceiver<Event>,
    events_sender: UnboundedSender<Event>,
    app_events: UnboundedSender<AppEvent>,
    timer: u64,
}

impl StateMachine {
    #[must_use]
    pub fn new(account_loaded: bool, app_events: UnboundedSender<AppEvent>) -> Self {
        let (sender, receiver) = unbounded_channel();

        Self {
            state: if account_loaded {
                AppState::Startup(Startup::default())
            } else {
                AppState::AccountMissing(AccountMissing::default())
            },
            cache: Cache::new(),
            stack: Vec::new(),
            events: receiver,
            events_sender: sender,
            app_events,
            timer: 0,
        }
    }

    #[allow(clippy::integer_arithmetic)]
    pub async fn tick(&mut self, account: &Option<Account>) {
        self.timer += 1;

        self.state
            .tick(account, self.timer, self.events_sender.clone())
            .await;

        let event = self.events.try_recv();

        if let Ok(e) = event {
            #[allow(clippy::single_match)]
            match &e {
                Event::Exited => {
                    let _result = self.app_events.send(AppEvent::Exit);
                }
                _ => {}
            }

            if let Some(transition) = self
                .state
                .transition(account, e, self.events_sender.clone())
            {
                match transition {
                    Transition::Push(state) => {
                        let old_state = std::mem::replace(&mut self.state, state);

                        let cache_index = self.cache.add(old_state);
                        self.stack.push(cache_index);
                    }
                    Transition::Pop => {
                        if let Some(index) = self.stack.pop() {
                            if let Some(state) = self.cache.get(index, &self.events_sender) {
                                // Don't cache these states (popups)
                                self.state = state;
                            }
                        }
                    }
                    Transition::To(state) => {
                        let old_state = std::mem::replace(&mut self.state, state);

                        let _cache_index = self.cache.add(old_state);
                    }
                }

                self.timer = 0;
            }
        };
    }

    pub fn render<B: Backend>(&mut self, theme: &Theme, frame: &mut Frame<'_, B>) {
        self.state.render(theme, frame, self.timer);
    }

    pub fn handle(&mut self, key_event: KeyEvent) {
        if let Some(action) = self.state.handle(key_event) {
            self.state.process(action, &self.events_sender);
        }
    }

    pub fn receive(&mut self) {
        self.state.receive();
    }
}
