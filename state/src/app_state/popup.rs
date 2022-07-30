mod choice;
mod timed_info;
mod user_input;

use crate::{
    app_state::exit::Exit,
    event::Event,
    state::{AppState, State},
    transition::Transition,
};
use async_trait::async_trait;
use choice::Choice;
use crossterm::event::KeyEvent;
use input::{handler::Handler, keybind::KeyBind};
use std::fmt::{Display, Formatter, Result};
use timed_info::TimedInfo;
use tokio::sync::mpsc::UnboundedSender;
use tui::{backend::Backend, terminal::Frame};
use twitch::account::Account;
use ui::{render, theme::Theme};
use user_input::Input;

pub enum Type {
    Choice(Choice),
    Input(Input),
    TimedInfo(TimedInfo),
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match *self {
            Type::Choice(_) => write!(f, "Choice"),
            Type::Input(_) => write!(f, "Input"),
            Type::TimedInfo(_) => write!(f, "Timed Info"),
        }
    }
}

// #[derive(Clone)]
// pub enum Output {
//     Empty,
//     Input(String),
//     Index(usize),
// }

pub struct Popup {
    title: String,
    message: String,
    pub variant: Type,
    input_handler: Handler<Event>,
}

// #[must_use]
// fn get_bool_choices() -> Vec<Choice> {
//     vec![
//         Choice {
//             display_text: String::from(BoolChoice::False.yes_no_display()),
//         },
//         Choice {
//             display_text: String::from(BoolChoice::True.yes_no_display()),
//         },
//     ]
// }

// #[must_use]
// pub fn get_chat_choice() -> Popup {
//     Popup {
//         title: String::from("Launch Chat"),
//         message: String::from("Do you want to launch the chat with the stream?"),
//         variant: PopupType::Choice {
//             selected: 0,
//             options: get_bool_choices(),
//         },
//     }
// }

impl Popup {
    pub fn new_choice(title: String, message: String, options: &[String]) -> Self {
        new(
            title,
            message,
            Type::Choice(Choice {
                selected: 0,
                options: options.to_vec(),
            }),
        )
    }

    pub fn new_input(title: String, message: String) -> Self {
        new(
            title,
            message,
            Type::Input(Input {
                typing: false,
                input: Vec::<char>::new(),
            }),
        )
    }

    pub fn new_timed_info(title: String, message: String, duration: u64) -> Self {
        new(title, message, Type::TimedInfo(TimedInfo { duration }))
    }
}

fn new(title: String, message: String, variant: Type) -> Popup {
    Popup {
        title,
        message,
        variant,
        input_handler: Handler::new(Vec::<KeyBind<Event>>::new()),
    }
}

#[async_trait]
impl State for Popup {
    fn keybinds(&self) -> Vec<KeyBind<Event>> {
        self.input_handler.inputs.clone()
    }

    async fn tick(&self, _: &Option<Account>, timer: u64, events: UnboundedSender<Event>) {
        if let Type::TimedInfo(popup) = &self.variant {
            if timer > popup.duration {
                let _result = events.send(Event::PopupEnded);
            }
        }
    }

    fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>, timer: u64) {
        match &self.variant {
            Type::Choice(choice) => render::choice(
                theme,
                frame,
                &self.keybinds(),
                &self.title,
                &self.message,
                choice.selected,
                &choice.options,
            ),
            Type::Input(input) => render::input(
                theme,
                frame,
                &self.keybinds(),
                &self.title,
                &self.message,
                &input.input,
                input.typing,
            ),
            Type::TimedInfo(timed_info) => render::timed_info(
                theme,
                frame,
                &self.keybinds(),
                &self.title,
                &self.message,
                timed_info.duration,
                timer,
            ),
        }
    }

    fn transition(
        &self,
        event: Event,
        _: &Option<Account>,
        _: UnboundedSender<Event>,
    ) -> Option<Transition> {
        match event {
            Event::Exited => Some(Transition::To(AppState::Exit(Exit::new()))),
            Event::PopupEnded => Some(Transition::Pop),
            _ => None,
        }
    }

    fn handle(&self, key_event: KeyEvent) -> Option<Event> {
        self.input_handler.handle(key_event)
    }

    fn process(&mut self, _: Event, _: &UnboundedSender<Event>) {}
}
