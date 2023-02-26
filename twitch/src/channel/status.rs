#![allow(clippy::use_self)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Status {
    Awaiting,
    Online,
    Offline,
    Unknown,
}

impl Default for Status {
    fn default() -> Self {
        Self::Awaiting
    }
}

impl Status {
    #[must_use]
    pub const fn message(&self) -> &'static str {
        match *self {
            Self::Awaiting => "...  ",
            Self::Online => "online",
            Self::Offline => "offline",
            Self::Unknown => "unknown",
        }
    }
}
