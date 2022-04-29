use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter, Result};

pub trait ExposeSecret {
    fn expose_value(&self) -> &str;
}

#[derive(Deserialize, Serialize)]
pub struct Secret {
    secret_string: String,
}

impl Secret {
    pub fn new(secret_string: String) -> Self {
        Self { secret_string }
    }
}

impl Debug for Secret {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "REDACTED")
    }
}

impl ExposeSecret for Secret {
    fn expose_value(&self) -> &str {
        &self.secret_string
    }
}
