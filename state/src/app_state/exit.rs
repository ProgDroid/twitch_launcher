use crate::{event::Event, state::State, transition::Transition};
use async_trait::async_trait;
use crossterm::event::KeyEvent;
use input::keybind::KeyBind;
use tokio::sync::mpsc::UnboundedSender;
use tui::{backend::Backend, terminal::Frame};
use twitch::account::Account;
use ui::theme::Theme;

pub struct Exit;

impl Exit {
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl State for Exit {
    fn keybinds(&self) -> Vec<KeyBind<Event>> {
        Vec::new()
    }

    async fn tick(&self, _: &Option<Account>, _: u64, _: UnboundedSender<Event>) {}

    fn render<B: Backend>(&self, _: &Theme, _: &mut Frame<'_, B>, _: u64) {}

    fn transition(
        &self,
        _: Event,
        _: &Option<Account>,
        _: UnboundedSender<Event>,
    ) -> Option<Transition> {
        None
    }

    fn handle(&self, _: KeyEvent) -> Option<Event> {
        None
    }

    fn process(&mut self, _: Event, _: &UnboundedSender<Event>) {}
}
