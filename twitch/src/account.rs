use crate::secret::{Expose, Secret};
use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, write};
use std::io::{Error, ErrorKind, Result};
use twitch_api2::twitch_oauth2::{
    refresh_token, AccessToken, ClientId, ClientSecret, RefreshToken, UserToken,
};

const ACCOUNT_FILE: &str = "account.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct Account {
    username: String,
    user_id: String,
    client_id: Secret,
    client_secret: Secret,
    user_access_token: Secret,
    refresh_token: Secret,
}

// TODO Allow authing on first run

impl Account {
    pub async fn load() -> Result<Self> {
        let data: String = read_to_string(ACCOUNT_FILE)?;

        let mut account: Self = serde_json::from_str(data.as_str())?;

        check_token(&mut account).await?;

        Ok(account)
    }

    pub fn save(&self) -> Result<()> {
        let file_contents: String = serde_json::to_string_pretty(&self)?;

        write(ACCOUNT_FILE, file_contents)?;
        Ok(())
    }

    #[must_use]
    pub fn access_token(&self) -> Secret {
        self.user_access_token.clone()
    }
}

async fn check_token(account: &mut Account) -> Result<()> {
    let existing_access_token =
        AccessToken::new(account.user_access_token.expose_value().to_owned());

    match UserToken::from_existing(
        &reqwest::Client::default(),
        existing_access_token,
        None,
        None,
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(_) => {
            match refresh_token(
                &reqwest::Client::default(),
                &RefreshToken::new(account.refresh_token.expose_value().to_owned()),
                &ClientId::new(account.client_id.expose_value().to_owned()),
                &ClientSecret::new(account.client_secret.expose_value().to_owned()),
            )
            .await
            {
                Ok((access_token, _, refresh_token)) => {
                    account.user_access_token = Secret::new(access_token.secret().to_owned());
                    account.refresh_token = match refresh_token {
                        Some(token) => Secret::new(token.secret().to_owned()),
                        None => Secret::new(account.refresh_token.expose_value().to_owned()),
                    };
                    account.save()?;

                    Ok(())
                }
                Err(_) => Err(Error::new(
                    ErrorKind::Other,
                    "Could not refresh token, invalid account could not be loaded",
                )),
            }
        }
    }
}
