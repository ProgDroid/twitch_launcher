mod choice;
mod timed_info;
mod user_input;

use crate::{
    app_state::exit::Exit,
    event::Event,
    input_mappings::{choice_inputs, timed_info_inputs, typing_inputs, user_input_inputs},
    state::{AppState, MoveDirection, MoveEnd, State},
    transition::Transition,
    util::{index_add, index_subtract},
};
use async_trait::async_trait;
use choice::Choice;
use crossterm::event::{KeyCode, KeyEvent};
use input::handler::Handler;
use std::fmt::{Display, Formatter, Result};
use timed_info::TimedInfo;
use tokio::sync::mpsc::UnboundedSender;
use tui::{backend::Backend, terminal::Frame};
use twitch::account::Account;
use ui::{render, theme::Theme};
use user_input::Input;

pub type Callback = fn(&UnboundedSender<Event>, &Output);

pub enum Type {
    Choice(Choice),
    Input(Input),
    TimedInfo(TimedInfo),
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match *self {
            Self::Choice(_) => write!(f, "Choice"),
            Self::Input(_) => write!(f, "Input"),
            Self::TimedInfo(_) => write!(f, "Timed Info"),
        }
    }
}

#[derive(Clone)]
pub enum Output {
    Input(String),
    Index(usize),
}

pub struct Popup {
    title: String,
    message: String,
    pub variant: Type,
    callback: Option<Callback>,
    input_handler: Handler<Event>,
}

impl Popup {
    pub fn new_choice(
        title: String,
        message: String,
        options: &[String],
        callback: Option<Callback>,
    ) -> Self {
        new(
            title,
            message,
            Type::Choice(Choice {
                selected: 0,
                options: options.to_vec(),
            }),
            callback,
        )
    }

    pub fn new_input(title: String, message: String, callback: Option<Callback>) -> Self {
        new(
            title,
            message,
            Type::Input(Input {
                typing: false,
                input: Vec::<char>::new(),
            }),
            callback,
        )
    }

    pub fn new_timed_info(
        title: String,
        message: String,
        duration: u64,
        callback: Option<Callback>,
    ) -> Self {
        new(
            title,
            message,
            Type::TimedInfo(TimedInfo { duration }),
            callback,
        )
    }
}

fn new(title: String, message: String, variant: Type, callback: Option<Callback>) -> Popup {
    let inputs = match &variant {
        Type::Choice(_) => choice_inputs(),
        Type::Input(popup) => {
            if popup.typing {
                typing_inputs()
            } else {
                user_input_inputs()
            }
        }
        Type::TimedInfo(_) => timed_info_inputs(),
    };

    Popup {
        title,
        message,
        variant,
        callback,
        input_handler: Handler::new(inputs),
    }
}

#[async_trait]
impl State for Popup {
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
                &self.input_handler.render(),
                &self.title,
                &self.message,
                choice.selected,
                &choice.options,
            ),
            Type::Input(input) => render::input(
                theme,
                frame,
                &self.input_handler.render(),
                &self.title,
                &self.message,
                &input.input,
                input.typing,
            ),
            Type::TimedInfo(timed_info) => render::timed_info(
                theme,
                frame,
                &self.input_handler.render(),
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
        let action = self.input_handler.handle(key_event);

        if let Type::Input(popup) = &self.variant {
            if popup.typing && action.is_none() {
                if let KeyCode::Char(char) = key_event.code {
                    return Some(Event::Typed(char));
                }
            }
        }

        action
    }

    fn process(&mut self, action: Event, tx: &UnboundedSender<Event>) {
        match action {
            Event::CycleHighlight(direction) => match self.variant {
                Type::Choice(ref mut popup) => {
                    popup.selected = match direction {
                        MoveDirection::Down => index_add(popup.selected, popup.options.len()),
                        MoveDirection::Up => index_subtract(popup.selected, popup.options.len()),
                        _ => popup.selected,
                    };
                }
                Type::Input(_) | Type::TimedInfo(_) => {}
            },
            Event::HomeEndHighlight(end) => match self.variant {
                Type::Choice(ref mut popup) => {
                    popup.selected = match end {
                        MoveEnd::First => 0,
                        MoveEnd::Last => popup.options.len() - 1,
                    };
                }
                Type::Input(_) | Type::TimedInfo(_) => {}
            },
            Event::Selected => match self.variant {
                Type::Choice(ref mut popup) => {
                    let _result = tx.send(Event::PopupEnded);

                    if let Some(func) = self.callback {
                        func(tx, &Output::Index(popup.selected));
                    }
                }
                Type::Input(ref mut popup) => {
                    popup.typing = true;
                    self.input_handler = Handler::new(typing_inputs());
                }
                Type::TimedInfo(_) => {}
            },
            Event::StopTyping => match self.variant {
                Type::Input(ref mut popup) => {
                    popup.typing = false;
                    self.input_handler = Handler::new(user_input_inputs());
                }
                Type::Choice(_) | Type::TimedInfo(_) => {}
            },
            Event::Submit => match self.variant {
                Type::Input(ref mut popup) => {
                    let input: String = popup.input.iter().collect();

                    if input.is_empty() {
                        return;
                    }

                    let _result = tx.send(Event::PopupEnded);

                    if let Some(func) = self.callback {
                        func(tx, &Output::Input(input));
                    }
                }
                Type::Choice(_) | Type::TimedInfo(_) => {}
            },
            Event::DeleteChar => match self.variant {
                Type::Input(ref mut popup) => {
                    popup.input.pop();
                }
                Type::Choice(_) | Type::TimedInfo(_) => {}
            },
            Event::Typed(char) => match self.variant {
                Type::Input(ref mut popup) => {
                    popup.input.push(char);
                }
                Type::Choice(_) | Type::TimedInfo(_) => {}
            },
            Event::Paste(to_paste) => match self.variant {
                Type::Input(ref mut popup) => {
                    for c in to_paste.chars() {
                        popup.input.push(c);
                    }
                }
                Type::Choice(_) | Type::TimedInfo(_) => {}
            },
            _ => {}
        }
    }
}

pub fn chat_choice(tx: &UnboundedSender<Event>, output: &Output) {
    if let Output::Index(choice) = output {
        let _result = tx.send(Event::ChatChoice(*choice));
    }
}

pub fn chat_choice_search(tx: &UnboundedSender<Event>, output: &Output) {
    if let Output::Index(choice) = output {
        let _result = tx.send(Event::ChatChoiceSearch(*choice));
    }
}

pub fn username_submit(tx: &UnboundedSender<Event>, output: &Output) {
    if let Output::Input(input) = output {
        let _result = tx.send(Event::SetUser(input.clone()));
    }
}

pub fn user_id_submit(tx: &UnboundedSender<Event>, output: &Output) {
    if let Output::Input(input) = output {
        let _result = tx.send(Event::SetUserId(input.clone()));
    }
}

pub fn client_id_submit(tx: &UnboundedSender<Event>, output: &Output) {
    if let Output::Input(input) = output {
        let _result = tx.send(Event::SetClientId(input.clone()));
    }
}

pub fn client_secret_submit(tx: &UnboundedSender<Event>, output: &Output) {
    if let Output::Input(input) = output {
        let _result = tx.send(Event::SetClientSecret(input.clone()));
    }
}

#[allow(clippy::expect_used)]
pub fn redirect_url_port_submit(tx: &UnboundedSender<Event>, output: &Output) {
    if let Output::Input(input) = output {
        let _result = tx.send(Event::SetRedirectUrlPort(
            input.parse().expect("Could not parse port"),
        ));
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn chat_popup(tx: &UnboundedSender<Event>) {
    let _result = tx.send(Event::ChoicePopupStarted((
        String::from("Launch Chat"),
        String::from("Do you want to launch the chat with the stream?"),
        vec![String::from("No"), String::from("Yes")],
        Some(chat_choice),
    )));
}

pub fn chat_popup_search(tx: &UnboundedSender<Event>) {
    let _result = tx.send(Event::ChoicePopupStarted((
        String::from("Launch Chat"),
        String::from("Do you want to launch the chat with the stream?"),
        vec![String::from("No"), String::from("Yes")],
        Some(chat_choice_search),
    )));
}
