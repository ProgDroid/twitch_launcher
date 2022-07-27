use crate::{
    app_state::{
        exit::Exit,
        home::Home,
        popup::Popup,
        startup::{AccountMissing, Startup},
    },
    event::Event,
    transition::Transition,
};
use async_trait::async_trait;
use crossterm::event::KeyEvent;
use input::keybind::KeyBind;
use tokio::sync::mpsc::UnboundedSender;
use tui::{backend::Backend, terminal::Frame};
use twitch::account::Account;
use ui::theme::Theme;

// handler::{keybinds_exit, keybinds_home, keybinds_popup, keybinds_startup, keybinds_typing},

#[async_trait]
pub trait State {
    fn keybinds(&self) -> Vec<KeyBind<Event>>;

    async fn tick(&self, account: &Option<Account>, timer: u64, events: UnboundedSender<Event>);

    fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>, timer: u64);

    fn transition(&self, event: Event, events_sender: UnboundedSender<Event>)
        -> Option<Transition>;

    fn handle(&self, key_event: KeyEvent);
}

// #[must_use]
// pub fn chat_choice(output: &Output) -> Event {
//     if let Output::Index(choice) = output {
//         return Event::ChatChoice(BoolChoice::from(*choice));
//     }

//     Event::ChatChoice(BoolChoice::False)
// }

#[allow(clippy::module_name_repetitions)]
pub enum AppState {
    AccountMissing(AccountMissing),
    Startup(Startup),
    Home(Home),
    Popup(Popup),
    // Follows(StateFollows),
    Exit(Exit),
}

impl AppState {
    pub async fn tick(&self, account: &Option<Account>, timer: u64, tx: UnboundedSender<Event>) {
        match self {
            Self::AccountMissing(state) => state.tick(account, timer, tx).await,
            Self::Startup(state) => state.tick(account, timer, tx).await,
            Self::Home(state) => state.tick(account, timer, tx).await,
            Self::Popup(state) => state.tick(account, timer, tx).await,
            Self::Exit(state) => state.tick(account, timer, tx).await,
        }
    }

    pub fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>, timer: u64) {
        match self {
            Self::AccountMissing(state) => state.render(theme, frame, timer),
            Self::Startup(state) => state.render(theme, frame, timer),
            Self::Home(state) => state.render(theme, frame, timer),
            Self::Popup(state) => state.render(theme, frame, timer),
            Self::Exit(state) => state.render(theme, frame, timer),
        }
    }

    #[must_use]
    pub fn transition(
        &self,
        event: Event,
        events_sender: UnboundedSender<Event>,
    ) -> Option<Transition> {
        match self {
            Self::AccountMissing(state) => state.transition(event, events_sender),
            Self::Startup(state) => state.transition(event, events_sender),
            Self::Home(state) => state.transition(event, events_sender),
            Self::Popup(state) => state.transition(event, events_sender),
            Self::Exit(state) => state.transition(event, events_sender),
        }
    }

    pub fn handle(&self, key_event: KeyEvent) {
        match self {
            Self::AccountMissing(state) => state.handle(key_event),
            Self::Startup(state) => state.handle(key_event),
            Self::Home(state) => state.handle(key_event),
            Self::Popup(state) => state.handle(key_event),
            Self::Exit(state) => state.handle(key_event),
        }
    }

    pub fn receive(&mut self, tx: &UnboundedSender<Event>) {
        match self {
            Self::Home(state) => state.channel_check(tx),
            Self::AccountMissing(_) | Self::Startup(_) | Self::Popup(_) | Self::Exit(_) => {}
        }
    }
}

// impl AppState {
//     fn keybinds(&self) -> Vec<KeyBind> {
//         match *self {
//             Self::Startup { .. } => keybinds_startup(),
//             Self::Home { typing, .. } => {
//                 if typing {
//                     keybinds_typing()
//                 } else {
//                     keybinds_home()
//                 }
//             }
//             Self::Popup { ref popup, .. } => {
//                 if let Popups::Input { typing, .. } = popup.variant {
//                     if typing {
//                         return keybinds_typing();
//                     }
//                 }

//                 keybinds_popup()
//             }
//             Self::Exit => keybinds_exit(),
//         }
//     }

//     #[allow(
//         clippy::integer_arithmetic,
//         clippy::indexing_slicing,
//         clippy::too_many_lines
//     )]
//     async fn tick(&mut self, events: &mut VecDeque<Event>) -> bool {
//         match self {
//             Self::Startup {
//                 account_loaded,
//                 ref mut timer,
//                 startup_duration,
//                 ref mut event_output,
//                 ref mut event_callback,
//                 ..
//             } => {
//                 *timer += 1;

//                 if *timer > (*startup_duration) {
//                     events.push_back(if *account_loaded {
//                         Event::AccountLoaded
//                     } else {
//                         Event::Exited
//                     });
//                 }

//                 if let Some(callback) = event_callback {
//                     match &event_output {
//                         Output::Index(_) | Output::String(_) => {
//                             events.push_back((callback)(event_output));
//                         }
//                         Output::Empty => {}
//                     };

//                     *event_output = Output::Empty;
//                     *event_callback = None;
//                 }

//                 // TODO startup account loading
//                 // let twitch_account: Option<TwitchAccount> = match TwitchAccount::load().await {
//                 //     Ok(account) => Some(account),
//                 //     Err(_) => None,
//                 // };

//                 // let account_loaded = twitch_account.is_some();

//                 true
//             }
//             Self::Home {
//                 twitch_account,
//                 channels,
//                 ref mut channel_check,
//                 ref mut event_output,
//                 ref mut event_callback,
//                 ..
//             } => {
//                 if let Some(check) = channel_check {
//                     while let Ok((handle, status)) = check.try_recv() {
//                         #[allow(clippy::expect_used)]
//                         let index: usize = channels
//                             .iter()
//                             .position(|channel| {
//                                 channel.handle == handle && channel.status != status
//                             })
//                             .expect("Received channel status for non-existing channel");
//                         channels[index].status = status;
//                     }

//                     if check.try_recv() == Err(TryRecvError::Disconnected) {
//                         check.close();
//                         *channel_check = None;
//                     }
//                 } else {
//                     let mut channels_awaiting: Vec<Channel> = Vec::new();

//                     for channel in channels {
//                         if channel.status == Status::Awaiting {
//                             channels_awaiting.push(channel.clone());
//                         }
//                     }

//                     if !channels_awaiting.is_empty() {
//                         #[allow(clippy::expect_used)]
//                         if let Ok((_, check)) = load_channels(
//                             twitch_account
//                                 .as_ref()
//                                 .expect("Made it past startup without loading Twitch account"),
//                             &channels_awaiting,
//                         ) {
//                             *channel_check = Some(check);
//                         }
//                     }
//                 }

//                 if let Some(callback) = event_callback {
//                     match event_output {
//                         Output::Index(_) | Output::String(_) => {
//                             events.push_back((callback)(event_output));

//                             *event_output = Output::Empty;
//                             *event_callback = None;
//                         }
//                         Output::Empty => {}
//                     }
//                 }

//                 true
//             }
//             Self::Popup { popup, .. } => {
//                 match popup.variant {
//                     Popups::TimedInfo {
//                         duration,
//                         ref mut timer,
//                         ..
//                     } => {
//                         *timer += 1;

//                         if *timer > (duration) {
//                             events.push_back(Event::PopupEnded(Output::Empty));
//                         }
//                     }
//                     Popups::Input { .. } | Popups::Choice { .. } => {}
//                 }

//                 true
//             }
//             Self::Exit => false,
//         }
//     }

//     fn render<B: Backend>(&self, theme: &Theme, frame: &mut Frame<'_, B>) {
//         match *self {
//             Self::Startup {
//                 account_loaded,
//                 timer,
//                 ..
//             } => {
//                 if account_loaded {
//                     render::startup_animation(theme, frame, &timer);
//                 } else {
//                     render::account_missing(theme, frame, &timer);
//                 }
//             }
//             Self::Home {
//                 tab,
//                 tab_titles,
//                 channel_highlight,
//                 ref channels,
//                 typing,
//                 ref search_input,
//                 ref focused_panel,
//                 ..
//             } => render::home(
//                 theme,
//                 frame,
//                 &tab,
//                 &tab_titles,
//                 &channel_highlight,
//                 channels,
//                 &typing,
//                 search_input,
//                 focused_panel,
//                 &self.keybinds(),
//             ),
//             Self::Popup { ref popup, .. } => render::popup(theme, frame, popup, &self.keybinds()),
//             Self::Exit { .. } => {}
//         }
//     }

//     #[allow(
//         clippy::unwrap_in_result,
//         clippy::too_many_lines
//     )]
//     // add `use self::Event::*;` here
//     fn transition(&self, event: Event) -> Option<Transition> {
//         match (self, event) {
//             (StateMachine::Startup { twitch_account, .. }, Event::AccountLoaded) => {
//                 #[allow(clippy::expect_used)]
//                 let account = twitch_account
//                     .as_ref()
//                     .expect("Made it past startup without loading Twitch account");

//                 let (channels, channel_check) = match load_channels_from_file(account) {
//                     Ok((channels, channel_check)) => (channels, channel_check),
//                     Err(e) => {
//                         eprintln!("Error loading channels: {}", e);
//                         return Some(Transition::To(Self::Exit));
//                     }
//                 };

//                 Some(Transition::To(Self::Home {
//                     tab: 0,
//                     tab_titles: TAB_TITLES,
//                     channel_highlight: 0,
//                     channels,
//                     twitch_account: Some(TwitchAccount {
//                         username: String::from(account.username.as_str()),
//                         user_id: String::from(account.user_id.as_str()),
//                         client_id: Secret::new(account.client_id.expose_value().to_owned()),
//                         client_secret: Secret::new(account.client_secret.expose_value().to_owned()),
//                         user_access_token: Secret::new(
//                             account.user_access_token.expose_value().to_owned(),
//                         ),
//                         refresh_token: Secret::new(account.refresh_token.expose_value().to_owned()),
//                     }),
//                     channel_check: Some(channel_check),
//                     typing: false,
//                     search_input: Vec::new(),
//                     focused_panel: Home::default(),
//                     event_output: Output::Empty,
//                     event_callback: None,
//                 }))
//             }
//             (Self::Home { .. }, Event::ChannelSelected(channel, chat)) => {
//                 if let Err(e) = channel.launch() {
//                     eprintln!("Error opening stream: {}", e);
//                 }

//                 if chat {
//                     if let Err(e) = channel.launch_chat() {
//                         eprintln!("Error opening chat: {}", e);
//                     }
//                 }

//                 None
//             }
//             #[allow(clippy::unnested_or_patterns)]
//             (Self::Home { .. }, Event::Exited) | (Self::Startup { .. }, Event::Exited) => {
//                 Some(Transition::To(Self::Exit))
//             }
//             (Self::Home { .. }, Event::PopupStart(popup)) => {
//                 Some(Transition::Push(Self::Popup { popup }))
//             }
//             (Self::Popup { .. }, Event::Exited) => Some(Transition::To(Self::Exit)),
//             (Self::Popup { .. }, Event::PopupEnded(_)) => Some(Transition::Pop),
//             (
//                 Self::Home {
//                     channels,
//                     channel_highlight,
//                     search_input,
//                     focused_panel,
//                     ..
//                 },
//                 Event::ChatChoice(choice),
//             ) => {
//                 let channel: Channel = match *focused_panel {
//                     Home::Favourites => {
//                         if let Some(channel) = channels.get(*channel_highlight) {
//                             (*channel).clone()
//                         } else {
//                             return None;
//                         }
//                     }
//                     Home::Search => {
//                         if search_input.is_empty() {
//                             return None;
//                         }

//                         let handle: String = search_input.iter().collect();

//                         // TODO allow setting to determine whether to launch on browser or locally
//                         Channel {
//                             friendly_name: String::new(),
//                             handle,
//                             status: Status::Unknown,
//                         }
//                     }
//                 };

//                 if let Err(e) = channel.launch() {
//                     eprintln!("Error opening stream: {}", e);
//                 }

//                 if choice.is_true() {
//                     if let Err(e) = channel.launch_chat() {
//                         eprintln!("Error opening chat: {}", e);
//                     }
//                 }

//                 None
//             }
//             _ => None,
//         }
//     }

//     fn clone(&mut self) -> Self {
//         match self {
//             Self::Startup {
//                 timer,
//                 startup_duration,
//                 event_output,
//                 event_callback,
//                 ..
//             } => Self::Startup {
//                 account_loaded: *account_loaded,
//                 timer: *timer,
//                 startup_duration: *startup_duration,
//                 twitch_account: (*twitch_account).clone(),
//                 event_output: (*event_output).clone(),
//                 event_callback: *event_callback,
//             },
//             Self::Home {
//                 tab,
//                 tab_titles,
//                 channel_highlight,
//                 channels,
//                 twitch_account,
//                 ref mut channel_check,
//                 typing,
//                 search_input,
//                 focused_panel,
//                 event_output,
//                 event_callback,
//                 ..
//             } => {
//                 if let Some(check) = channel_check {
//                     check.close();
//                 }

//                 Self::Home {
//                     tab: *tab,
//                     tab_titles: *tab_titles,
//                     channel_highlight: *channel_highlight,
//                     channels: (*channels).clone(),
//                     twitch_account: (*twitch_account).clone(),
//                     channel_check: None,
//                     typing: *typing,
//                     search_input: (*search_input).clone(),
//                     focused_panel: *focused_panel,
//                     event_output: (*event_output).clone(),
//                     event_callback: *event_callback,
//                 }
//             }
//             Self::Popup { popup } => Self::Popup {
//                 popup: (*popup).clone(),
//             },
//             Self::Exit => Self::Exit,
//         }
//     }

//     // fn get_event_output(&self) -> &Output {
//     //     match *self {
//     //         Self::Startup {
//     //             ref event_output, ..
//     //         }
//     //         | Self::Home {
//     //             ref event_output, ..
//     //         } => event_output,
//     //         _ => &Output::Empty,
//     //     }
//     // }

//     // fn set_event_output(&mut self, output: Output) {
//     //     match *self {
//     //         Self::Startup {
//     //             ref mut event_output,
//     //             ..
//     //         }
//     //         | Self::Home {
//     //             ref mut event_output,
//     //             ..
//     //         } => *event_output = output,
//     //         _ => {}
//     //     }
//     // }
// }

// TODO state enter and exit?

// TODO move to utils
// #[allow(clippy::integer_arithmetic, clippy::integer_division)]
// const fn ticks_from_seconds(tick_rate: u64, seconds: u64) -> u64 {
//     (1000_u64 / tick_rate) * seconds
// }
