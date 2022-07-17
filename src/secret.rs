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
    pub const fn new(secret_string: String) -> Self {
        Self { secret_string }
    }
}

impl Debug for Secret {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "REDACTED")
    }
}

impl Expose for Secret {
    fn expose_value(&self) -> &str {
        &self.secret_string
    }
}
