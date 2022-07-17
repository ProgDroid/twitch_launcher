use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter, Result};

pub trait Expose {
    fn expose_value(&self) -> &str;
}

#[derive(Deserialize, Serialize)]
pub struct Secret {
    secret_string: String,
}

impl Secret {
    #[must_use]
    #[allow(clippy::missing_inline_in_public_items)]
    pub const fn new(secret_string: String) -> Self {
        Self { secret_string }
    }
}

impl Debug for Secret {
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "REDACTED")
    }
}

impl Expose for Secret {
    #[allow(clippy::missing_inline_in_public_items)]
    fn expose_value(&self) -> &str {
        &self.secret_string
    }
}
