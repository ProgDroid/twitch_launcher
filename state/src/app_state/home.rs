use crate::{
    app_state::{
        exit::Exit,
        popup::{chat_popup, chat_popup_search, Popup},
    },
    event::Event,
    input_mappings::{home_inputs, typing_inputs},
    state::{AppState, MoveDirection, MoveEnd, State},
    transition::Transition,
    util::{index_add, index_subtract},
};
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use input::handler::Handler;
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

use super::lists::Lists;

pub struct Home {
    channel_highlight: usize,
    pub favourites: Vec<Channel>,
    channel_check: UnboundedReceiver<(String, (Status, Option<String>))>,
    channel_check_sender: UnboundedSender<(String, (Status, Option<String>))>,
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
        _: &UnboundedSender<Event>,
    ) -> Self {
        let (sender, receiver) = unbounded_channel();

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

    pub fn from_existing(state: &mut Self, tx: &UnboundedSender<Event>) -> Self {
        let mut channels = state.favourites.clone();

        while let Ok((handle, (status, game_name))) = state.channel_check.try_recv() {
            if let Some(index) = state
                .favourites
                .iter()
                .position(|channel| channel.handle == handle && channel.status != status)
            {
                channels[index].status = status;
                channels[index].game = game_name;
            }
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
        let mut channels_awaiting: Vec<Channel> = Vec::new();

        for channel in favourites {
            if channel.status == Status::Awaiting {
                channels_awaiting.push(channel.clone());
            }
        }

        if !channels_awaiting.is_empty() {
            let _result = tx.send(Event::CheckChannels(channels_awaiting));
        }

        Self::new(0, favourites, false, &Vec::new(), HomePanel::default(), tx)
    }

    pub fn channel_check(&mut self) {
        let mut channels = self.favourites.clone();

        while let Ok((handle, (status, game_name))) = self.channel_check.try_recv() {
            if let Some(index) = self
                .favourites
                .iter()
                .position(|channel| channel.handle == handle && channel.status != status)
            {
                channels[index].status = status;
                channels[index].game = game_name;
            }
        }

        self.favourites = channels;
    }
}

#[async_trait]
impl State for Home {
    #[allow(clippy::ignored_unit_patterns)]
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
            &self.input_handler.render(),
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
            Event::ChoicePopupStarted((title, message, options, callback)) => {
                Some(Transition::Push(AppState::Popup(Popup::new_choice(
                    title, message, &options, callback,
                ))))
            }
            Event::InputPopupStarted((title, message, callback)) => Some(Transition::Push(
                AppState::Popup(Popup::new_input(title, message, callback)),
            )),
            Event::TimedInfoPopupStarted((title, message, duration, callback)) => {
                Some(Transition::Push(AppState::Popup(Popup::new_timed_info(
                    title, message, duration, callback,
                ))))
            }
            Event::ChatChoice(choice) => {
                let chat_choice = choice == 1;

                if let Some(channel) = self.favourites.get(self.channel_highlight) {
                    let _result = tx.send(Event::ChannelSelected((*channel).clone(), chat_choice));
                }

                None
            }
            Event::ChatChoiceSearch(choice) => {
                let chat_choice = choice == 1;

                // TODO check if channel exists?
                // TODO check if channel is online?
                let handle: String = self.search_input.iter().collect();

                let channel = Channel::new(handle.clone(), handle, None);

                let _result = tx.send(Event::ChannelSelected(channel, chat_choice));

                None
            }
            Event::CycleTab(direction) => match direction {
                MoveDirection::Left | MoveDirection::Right => {
                    // TODO shouldn't be reloading this every time
                    Some(Transition::To(AppState::Lists(Lists::init(&tx))))
                }
                _ => None,
            },
            Event::ChannelSelected(channel, chat) => {
                if let Err(e) = channel.launch() {
                    eprintln!("Error opening stream: {e}");
                }

                if chat {
                    if let Err(e) = channel.launch_chat() {
                        eprintln!("Error opening chat: {e}");
                    }
                }

                None
            }
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
                if self.focused_panel == HomePanel::Favourites {
                    self.channel_highlight = match direction {
                        MoveDirection::Down => {
                            index_add(self.channel_highlight, self.favourites.len())
                        }
                        MoveDirection::Up => {
                            index_subtract(self.channel_highlight, self.favourites.len())
                        }
                        _ => self.channel_highlight,
                    };
                }
            }
            Event::HomeEndHighlight(end) => {
                if self.focused_panel == HomePanel::Favourites {
                    self.channel_highlight = match end {
                        MoveEnd::First => 0,
                        MoveEnd::Last => self.favourites.len() - 1,
                    };
                }
            }
            Event::Selected => match self.focused_panel {
                HomePanel::Favourites => {
                    if self.favourites.get(self.channel_highlight).is_some() {
                        chat_popup(tx);
                    }
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
                if self.search_input.is_empty() {
                    return;
                }

                self.typing = false;
                self.input_handler = Handler::new(home_inputs());
                chat_popup_search(tx);
            }
            Event::DeleteChar => {
                self.search_input.pop();
            }
            Event::Typed(char) => {
                self.search_input.push(char);
            }
            Event::Paste(to_paste) => {
                for c in to_paste.chars() {
                    self.search_input.push(c);
                }
            }
            _ => {}
        }
    }
}
