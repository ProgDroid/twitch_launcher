use crate::secret::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, write};
use std::io::{Error, ErrorKind, Result};
use twitch_api2::twitch_oauth2::{
    refresh_token, AccessToken, ClientId, ClientSecret, RefreshToken, UserToken,
};

const ACCOUNT_FILE: &str = "account.json";

// TODO look into TwitchToken

#[derive(Serialize, Deserialize, Debug)]
pub struct TwitchAccount {
    pub username: String,
    pub user_id: String,
    pub client_id: Secret,
    pub client_secret: Secret,
    pub user_access_token: Secret,
    pub refresh_token: Secret,
}

impl TwitchAccount {
    pub async fn load() -> Result<Self> {
        let data: String = read_to_string(ACCOUNT_FILE)?;

        let mut account: TwitchAccount = serde_json::from_str(data.as_str())?;

        check_token(&mut account).await?;

        Ok(account)
    }

    pub fn save(self: &Self) -> Result<()> {
        let file_contents: String = serde_json::to_string_pretty(&self)?;

        write(ACCOUNT_FILE, file_contents)?;
        Ok(())
    }
}

async fn check_token(twitch_account: &mut TwitchAccount) -> Result<()> {
    let token = AccessToken::new(twitch_account.user_access_token.expose_value().to_string());

    match UserToken::from_existing(&reqwest::Client::default(), token, None, None).await {
        Ok(_) => Ok(()),
        Err(_) => {
            match refresh_token(
                &reqwest::Client::default(),
                &RefreshToken::new(twitch_account.refresh_token.expose_value().to_string()),
                &ClientId::new(twitch_account.client_id.expose_value().to_string()),
                &ClientSecret::new(twitch_account.client_secret.expose_value().to_string()),
            )
            .await
            {
                Ok((access_token, _, refresh_token)) => {
                    twitch_account.user_access_token =
                        Secret::new(access_token.secret().to_owned());
                    twitch_account.refresh_token = match refresh_token {
                        Some(token) => Secret::new(token.secret().to_owned()),
                        None => {
                            Secret::new(twitch_account.refresh_token.expose_value().to_string())
                        }
                    };
                    twitch_account.save()?;

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
