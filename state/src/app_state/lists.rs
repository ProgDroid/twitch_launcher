use std::fs::read_to_string;

use async_trait::async_trait;
use crossterm::event::KeyEvent;
use input::handler::Handler;
use jwalk::WalkDir;
use tokio::sync::mpsc::UnboundedSender;
use tui::{backend::Backend, terminal::Frame};
use twitch::{
    account::Account,
    channel::{Channel, List},
};
use ui::{render, theme::Theme};

use crate::{
    app_state::{exit::Exit, home::Home},
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
}

impl Lists {
    pub fn new(highlight: usize, lists: &[List]) -> Self {
        Self {
            lists: lists.to_vec(),
            highlight,
            input_handler: Handler::new(lists_inputs()),
        }
    }

    pub fn from_existing(state: &mut Self) -> Self {
        Self::new(state.highlight, &state.lists.clone())
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

        Self::new(0, &lists)
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
        );
    }

    fn transition(
        &self,
        event: Event,
        _: &Option<Account>,
        tx: UnboundedSender<Event>,
    ) -> Option<Transition> {
        match event {
            Event::Exited => Some(Transition::To(AppState::Exit(Exit::new()))),
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
            Event::CycleHighlight(direction) => {
                self.highlight = match direction {
                    MoveDirection::Down => index_add(self.highlight, self.lists.len()),
                    MoveDirection::Up => index_subtract(self.highlight, self.lists.len()),
                    _ => self.highlight,
                };
            }
            Event::HomeEndHighlight(end) => {
                self.highlight = match end {
                    MoveEnd::First => 0,
                    MoveEnd::Last => self.lists.len() - 1,
                };
            }
            _ => {}
        }
    }
}
