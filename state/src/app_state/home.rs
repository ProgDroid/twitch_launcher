use crate::{
    app_state::{exit::Exit, popup::Popup},
    event::Event,
    input_mappings::{home_inputs, typing_inputs},
    state::{AppState, MoveDirection, MoveEnd, State},
    transition::Transition,
    util::{index_add, index_subtract},
};
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use input::{handler::Handler, keybind::KeyBind};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tui::{backend::Backend, terminal::Frame};
use twitch::{
    account::Account,
    channel::{status::Status, Channel},
};
use ui::{
    panel::{Home as HomePanel, Panel},
    render,
    theme::Theme,
};

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
    pub fn new(
        channel_highlight: usize,
        favourites: &[Channel],
        typing: bool,
        search_input: &[char],
        focused_panel: HomePanel,
        tx: &UnboundedSender<Event>,
    ) -> Self {
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

        let inputs = if typing {
            typing_inputs()
        } else {
            home_inputs()
        };

        Self {
            channel_highlight,
            favourites: favourites.to_vec(),
            channel_check: receiver,
            channel_check_sender: sender,
            typing,
            search_input: search_input.to_vec(),
            focused_panel,
            input_handler: Handler::new(inputs),
        }
    }

    #[allow(clippy::indexing_slicing)]
    pub fn from_existing(state: &mut Self, tx: &UnboundedSender<Event>) -> Self {
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

        Self::new(
            state.channel_highlight,
            &channels,
            state.typing,
            &state.search_input,
            state.focused_panel,
            tx,
        )
    }

    pub fn init(favourites: &[Channel], tx: &UnboundedSender<Event>) -> Self {
        Self::new(0, favourites, false, &Vec::new(), HomePanel::default(), tx)
    }

    #[allow(clippy::indexing_slicing)]
    pub fn channel_check(&mut self) {
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

        self.favourites = channels;
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

    #[allow(clippy::too_many_lines)]
    fn transition(
        &self,
        event: Event,
        account: &Option<Account>,
        tx: UnboundedSender<Event>,
    ) -> Option<Transition> {
        match event {
            Event::Exited => Some(Transition::To(AppState::Exit(Exit::new()))),
            Event::CheckChannels(channels) => {
                if let Some(acc) = account {
                    Channel::check(&channels, acc, &self.channel_check_sender);
                }

                None
            }
            Event::ChoicePopupStarted((title, message, options)) => Some(Transition::Push(
                AppState::Popup(Popup::new_choice(title, message, &options)),
            )),
            Event::InputPopupStarted((title, message)) => Some(Transition::Push(AppState::Popup(
                Popup::new_input(title, message),
            ))),
            Event::TimedInfoPopupStarted((title, message, duration)) => Some(Transition::Push(
                AppState::Popup(Popup::new_timed_info(title, message, duration)),
            )),
            #[allow(clippy::collapsible_else_if)]
            Event::ChatChoice(choice) => {
                if self.typing && !self.search_input.is_empty() {
                    // TODO check if channel exists?
                    // TODO check if channel is online?
                    let handle: String = self.search_input.iter().collect();

                    let channel = Channel::new(handle.clone(), handle);

                    let _result = tx.send(Event::ChannelSelected(channel, choice));
                } else {
                    if let Some(channel) = self.favourites.get(self.channel_highlight) {
                        let _result = tx.send(Event::ChannelSelected((*channel).clone(), choice));
                    }
                }

                None
            }
            Event::CycleTab(_direction) => None,
            _ => None,
        }
    }

    fn handle(&self, key_event: KeyEvent) -> Option<Event> {
        let action = self.input_handler.handle(key_event);

        if self.typing && action.is_none() {
            if let KeyCode::Char(char) = key_event.code {
                return Some(Event::Typed(char));
            }
        }

        action
    }

    fn process(&mut self, action: Event, tx: &UnboundedSender<Event>) {
        match action {
            Event::Exited | Event::CycleTab(_) => {
                let _result = tx.send(action);
            }
            Event::CycleHighlight(direction) => {
                self.channel_highlight = match direction {
                    MoveDirection::Down => index_add(self.channel_highlight, self.favourites.len()),
                    MoveDirection::Up => {
                        index_subtract(self.channel_highlight, self.favourites.len())
                    }
                    _ => self.channel_highlight,
                };
            }
            Event::HomeEndHighlight(end) => {
                self.channel_highlight = match end {
                    MoveEnd::First => 0,
                    MoveEnd::Last => self.favourites.len(),
                };
            }
            Event::Selected => match self.focused_panel {
                HomePanel::Favourites => {
                    chat_popup(tx);
                }
                HomePanel::Search => {
                    self.typing = true;
                    self.input_handler = Handler::new(typing_inputs());
                }
            },
            Event::CyclePanel(direction) => {
                self.focused_panel = match direction {
                    MoveDirection::Left => self.focused_panel.left(),
                    MoveDirection::Right => self.focused_panel.right(),
                    _ => self.focused_panel,
                };
            }
            Event::StopTyping => {
                self.typing = false;
                self.input_handler = Handler::new(home_inputs());
            }
            Event::Submit => {
                chat_popup(tx);
            }
            Event::DeleteChar => {
                self.search_input.pop();
            }
            Event::Typed(char) => {
                self.search_input.push(char);
            }
            _ => {}
        }
    }
}

fn chat_popup(tx: &UnboundedSender<Event>) {
    let _result = tx.send(Event::ChoicePopupStarted((
        String::from("Launch Chat"),
        String::from("Do you want to launch the chat with the stream?"),
        vec![String::from("No"), String::from("Yes")],
    )));
}