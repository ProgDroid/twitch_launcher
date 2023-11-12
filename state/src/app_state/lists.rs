use std::fs::read_to_string;

use async_trait::async_trait;
use crossterm::event::KeyEvent;
use input::handler::Handler;
use jwalk::WalkDir;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tui::{backend::Backend, terminal::Frame};
use twitch::{
    account::Account,
    channel::{status::Status, Channel, List},
};
use ui::{
    panel::{Lists as ListsPanel, Panel},
    render,
    theme::Theme,
};

use crate::{
    app_state::{
        exit::Exit,
        home::Home,
        popup::{chat_popup, Popup},
    },
    event::Event,
    input_mappings::lists_inputs,
    state::{AppState, MoveDirection, MoveEnd, State},
    transition::Transition,
    util::{index_add, index_subtract},
};

const CHANNELS_FILE: &str = "favourites.json";
const LISTS_PATH: &str = "lists";
const LISTS_FILE_EXTENSION: &str = ".json";

pub struct Lists {
    lists: Vec<List>,
    highlight: usize,
    input_handler: Handler<Event>,
    focused_panel: ListsPanel,
    open_list: Option<usize>,
    channel_highlight: usize,
    channel_check: UnboundedReceiver<(String, (Status, Option<String>))>,
    channel_check_sender: UnboundedSender<(String, (Status, Option<String>))>,
}

impl Lists {
    pub fn new(
        highlight: usize,
        lists: &[List],
        open_list: Option<usize>,
        channel_highlight: usize,
    ) -> Self {
        let (sender, receiver) = unbounded_channel();

        Self {
            lists: lists.to_vec(),
            highlight,
            input_handler: Handler::new(lists_inputs()),
            focused_panel: ListsPanel::default(),
            open_list,
            channel_highlight,
            channel_check: receiver,
            channel_check_sender: sender,
        }
    }

    pub fn from_existing(state: &mut Self) -> Self {
        let mut updated_lists = state.lists.clone();

        for (index, list) in state.lists.iter_mut().enumerate() {
            let mut channels = list.channels.clone();

            while let Ok((handle, (status, game_name))) = state.channel_check.try_recv() {
                #[allow(clippy::expect_used)]
                let index: usize = list
                    .channels
                    .iter()
                    .position(|channel| channel.handle == handle && channel.status != status)
                    .expect("Received channel status for non-existing channel");
                channels[index].status = status;
                channels[index].game = game_name;
            }

            updated_lists[index].channels = channels;
        }

        Self::new(
            state.highlight,
            &updated_lists,
            state.open_list,
            state.channel_highlight,
        )
    }

    pub fn init() -> Self {
        let lists = WalkDir::new(LISTS_PATH)
            .sort(true)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name
                    .to_str()
                    .unwrap_or("")
                    .ends_with(LISTS_FILE_EXTENSION)
            })
            .map(|entry| (entry.path(), entry.file_name.into_string().ok()))
            .map(|(file_path, file_name)| {
                // TODO error handling here needs doing
                let data: String = read_to_string(file_path).unwrap_or_default();

                let channels = serde_json::from_str(data.as_str()).unwrap_or_default();

                let name = file_name.unwrap_or_default();

                List {
                    channels,
                    name: name.replace(LISTS_FILE_EXTENSION, ""),
                    path: name,
                }
            })
            .collect::<Vec<List>>();

        Self::new(0, &lists, None, 0)
    }

    pub fn channel_check(&mut self) {
        if let Some(open_list_index) = self.open_list {
            if let Some(list) = self.lists.get(open_list_index) {
                let mut channels = list.channels.clone();

                while let Ok((handle, (status, game_name))) = self.channel_check.try_recv() {
                    #[allow(clippy::expect_used)]
                    let index: usize = channels
                        .iter()
                        .position(|channel| channel.handle == handle && channel.status != status)
                        .expect("Received channel status for non-existing channel");
                    channels[index].status = status;
                    channels[index].game = game_name;
                }

                self.lists[open_list_index].channels = channels;
            }
        }
    }
}

#[async_trait]
impl State for Lists {
    async fn tick(&self, _: &Option<Account>, _: u64, _: UnboundedSender<Event>) {}

    fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>, _: u64) {
        render::lists(
            theme,
            frame,
            self.highlight,
            &self.lists,
            &self.input_handler.render(),
            self.focused_panel,
            self.open_list,
            self.channel_highlight,
        );
    }

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

                if let Some(list_index) = self.open_list {
                    if let Some(list) = self.lists.get(list_index) {
                        if let Some(channel) = list.channels.get(self.channel_highlight) {
                            let _result =
                                tx.send(Event::ChannelSelected((*channel).clone(), chat_choice));
                        }
                    }
                }

                None
            }
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
            Event::CycleTab(direction) => match direction {
                MoveDirection::Left | MoveDirection::Right => {
                    // TODO shouldn't be reloading this every time
                    let channels = Channel::load_from_file(CHANNELS_FILE)
                        .map_or_else(|_| Vec::new(), |channels| channels);

                    Some(Transition::To(AppState::Home(Home::init(
                        channels.as_slice(),
                        &tx,
                    ))))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn handle(&self, key_event: KeyEvent) -> Option<Event> {
        self.input_handler.handle(key_event)
    }

    fn process(&mut self, action: Event, tx: &UnboundedSender<Event>) {
        match action {
            Event::Exited | Event::CycleTab(_) => {
                let _result = tx.send(action);
            }
            Event::CycleHighlight(direction) => match self.focused_panel {
                ListsPanel::Lists => {
                    self.highlight = match direction {
                        MoveDirection::Down => index_add(self.highlight, self.lists.len()),
                        MoveDirection::Up => index_subtract(self.highlight, self.lists.len()),
                        _ => self.highlight,
                    };
                }
                ListsPanel::ListContent => {
                    if let Some(open_list_index) = self.open_list {
                        if let Some(list) = self.lists.get(open_list_index) {
                            self.channel_highlight = match direction {
                                MoveDirection::Down => {
                                    index_add(self.channel_highlight, list.channels.len())
                                }
                                MoveDirection::Up => {
                                    index_subtract(self.channel_highlight, list.channels.len())
                                }
                                _ => self.channel_highlight,
                            };
                        }
                    }
                }
            },
            Event::HomeEndHighlight(end) => {
                self.highlight = match end {
                    MoveEnd::First => 0,
                    MoveEnd::Last => self.lists.len() - 1,
                };
            }
            Event::Selected => match self.focused_panel {
                ListsPanel::Lists => {
                    self.open_list = Some(self.highlight);
                    self.channel_highlight = 0;
                }
                ListsPanel::ListContent => {
                    chat_popup(tx);
                }
            },
            Event::CyclePanel(direction) => {
                self.focused_panel = match direction {
                    MoveDirection::Left => self.focused_panel.left(),
                    MoveDirection::Right => match self.open_list {
                        Some(_) => self.focused_panel.right(),
                        None => self.focused_panel,
                    },
                    _ => self.focused_panel,
                };
            }
            _ => {}
        }
    }
}
