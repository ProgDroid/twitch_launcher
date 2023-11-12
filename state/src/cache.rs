use crate::{
    app_state::{
        home::Home,
        lists::Lists,
        startup::{AccountMissing, Startup},
    },
    event::Event,
    state::AppState,
};
use std::mem::discriminant;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub struct Cache {
    storage: Vec<AppState>,
}

// TODO turn into channel status cache
// TODO could be more useful, only used for popups

impl Cache {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            storage: Vec::new(),
        }
    }

    pub fn add(&mut self, state: AppState) -> usize {
        if let Some(index) = self
            .storage
            .iter()
            .position(|cached_state| discriminant(&state) == discriminant(cached_state))
        {
            if let Some(existing_state) = self.storage.get_mut(index) {
                *existing_state = state;
                return index;
            }

            eprintln!("State found in cache but could not be grabbed");
        }

        self.storage.push(state);
        self.storage.len() - 1
    }

    pub fn get(&mut self, index: usize, tx: &UnboundedSender<Event>) -> Option<AppState> {
        if let Some(state) = self.storage.get_mut(index) {
            match state {
                AppState::AccountMissing(s) => {
                    return Some(AppState::AccountMissing(AccountMissing::new(
                        s.timer,
                        s.duration,
                        s.account_config.clone(),
                    )));
                }
                AppState::Startup(s) => {
                    return Some(AppState::Startup(Startup::new(s.timer, s.duration)));
                }
                AppState::Home(s) => return Some(AppState::Home(Home::from_existing(s, tx))),
                AppState::Lists(s) => return Some(AppState::Lists(Lists::from_existing(s))),
                AppState::Popup(_) | AppState::Exit(_) => return None, // Not cached
            }
        }

        None
    }
}
