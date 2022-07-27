use crate::{
    app_state::{exit::Exit, popup::Popup},
    event::Event,
    input_mappings::home_inputs,
    state::{AppState, State},
    transition::Transition,
};
use async_trait::async_trait;
use crossterm::event::KeyEvent;
use input::{handler::Handler, keybind::KeyBind};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tui::{backend::Backend, terminal::Frame};
use twitch::{
    account::Account,
    channel::{status::Status, Channel},
};
use ui::{panel::Home as HomePanel, render, theme::Theme};

#[allow(dead_code)]
pub struct Home {
    channel_highlight: usize,
    pub favourites: Vec<Channel>,
    channel_check: UnboundedReceiver<(String, Status)>,
    channel_check_sender: UnboundedSender<(String, Status)>,
    typing: bool,
    search_input: Vec<char>,
    focused_panel: HomePanel,
    input_handler: Handler<Event>,
}

impl Home {
    pub fn new(favourites: &[Channel], tx: UnboundedSender<Event>) -> Self {
        let (sender, receiver) = unbounded_channel();

        let mut channels_awaiting: Vec<Channel> = Vec::new();

        for channel in favourites {
            if channel.status == Status::Awaiting {
                channels_awaiting.push(channel.clone());
            }
        }

        if !channels_awaiting.is_empty() {
            let _result = tx.send(Event::CheckChannels(channels_awaiting));
        }

        Self {
            channel_highlight: 0,
            favourites: favourites.to_vec(),
            channel_check: receiver,
            channel_check_sender: sender,
            typing: false,
            search_input: Vec::new(),
            focused_panel: HomePanel::default(),
            input_handler: Handler::new(tx, home_inputs()),
        }
    }

    #[allow(clippy::indexing_slicing)]
    pub fn from(state: &mut Self, tx: UnboundedSender<Event>) -> Self {
        let mut channels = state.favourites.clone();

        while let Ok((handle, status)) = state.channel_check.try_recv() {
            #[allow(clippy::expect_used)]
            let index: usize = state
                .favourites
                .iter()
                .position(|channel| channel.handle == handle && channel.status != status)
                .expect("Received channel status for non-existing channel");
            channels[index].status = status;
        }

        let mut home = Self::new(&channels, tx);

        home.channel_highlight = state.channel_highlight;
        home.typing = state.typing;
        home.search_input = state.search_input.clone();
        home.focused_panel = state.focused_panel;

        home
    }

    #[allow(clippy::indexing_slicing)]
    pub fn channel_check(&mut self, events: &UnboundedSender<Event>) {
        let mut channels = self.favourites.clone();

        while let Ok((handle, status)) = self.channel_check.try_recv() {
            #[allow(clippy::expect_used)]
            let index: usize = self
                .favourites
                .iter()
                .position(|channel| channel.handle == handle && channel.status != status)
                .expect("Received channel status for non-existing channel");
            channels[index].status = status;
        }

        let _result = events.send(Event::FavouritesLoaded(channels));
    }
}

#[async_trait]
impl State for Home {
    fn keybinds(&self) -> Vec<KeyBind<Event>> {
        self.input_handler.inputs.clone()
    }

    async fn tick(&self, _: &Option<Account>, _: u64, _: UnboundedSender<Event>) {}

    fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>, _: u64) {
        render::home(
            theme,
            frame,
            &self.channel_highlight,
            &self.favourites,
            self.typing,
            &self.search_input,
            &self.focused_panel,
            &self.keybinds(),
        );
    }

    fn transition(&self, event: Event, tx: UnboundedSender<Event>) -> Option<Transition> {
        match event {
            Event::ChannelSelected(channel, chat) => {
                if let Err(e) = channel.launch() {
                    eprintln!("Error opening stream: {}", e);
                }

                if chat {
                    if let Err(e) = channel.launch_chat() {
                        eprintln!("Error opening chat: {}", e);
                    }
                }

                None
            }
            Event::Exited => Some(Transition::To(AppState::Exit(Exit::new()))),
            Event::ChoicePopupStarted((title, message, options)) => Some(Transition::Push(
                AppState::Popup(Popup::new_choice(title, message, &options, tx)),
            )),
            _ => None,
        }
    }

    fn handle(&self, key_event: KeyEvent) {
        self.input_handler.handle(key_event);
    }
}

// TODO add check favourites function and force check favourites function
