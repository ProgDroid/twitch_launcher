use crate::secret::{Expose, Secret};
use anyhow::{Context, Result};
use open;
use serde::{Deserialize, Serialize};
use server::server::Server;
use std::{
    fs::{copy, read_to_string, write},
    path::Path,
};
use twitch_api::twitch_oauth2::{
    AccessToken, ClientId, ClientSecret, RefreshToken, TwitchToken, UserToken,
};

const ACCOUNT_FILE: &str = "account.json";
const ACCOUNT_DIST_FILE: &str = "account.json.dist";

#[must_use]
#[derive(Serialize, Deserialize, Clone)]
pub struct Account {
    username: String,
    user_id: String,
    client_id: Secret,
    client_secret: Secret,
    user_access_token: Secret,
    refresh_token: Secret,
    redirect_url_port: u16,
}

impl Account {
    #[allow(clippy::missing_errors_doc)]
    pub async fn load() -> Result<Self> {
        if !Path::new(ACCOUNT_FILE).exists() {
            copy(ACCOUNT_DIST_FILE, ACCOUNT_FILE)?;
        }

        let mut account: Self = load_account()?;

        check_token(&mut account).await?;

        Ok(account)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn save(&self) -> Result<()> {
        let file_contents: String = serde_json::to_string_pretty(&self)?;

        write(ACCOUNT_FILE, file_contents)?;
        Ok(())
    }

    #[must_use]
    pub fn access_token(&self) -> Secret {
        self.user_access_token.clone()
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn new(
        username: String,
        user_id: String,
        client_id: String,
        client_secret: String,
        port: u16,
    ) -> Result<Self> {
        let mut account = Self {
            username,
            user_id,
            client_id: Secret::new(client_id),
            client_secret: Secret::new(client_secret),
            user_access_token: Secret::new(String::new()),
            refresh_token: Secret::new(String::new()),
            redirect_url_port: port,
        };

        let server = Server::new(account.redirect_url_port)?;

        let url = server.redirect_url()?;

        let mut builder = UserToken::builder(
            ClientId::new(account.client_id.expose_value().to_owned()),
            ClientSecret::new(account.client_secret.expose_value().to_owned()),
            url,
        );

        let (auth_url, _) = builder.generate_url();

        open::that(auth_url.as_str())?;

        let (code, state) = server.get_callback_data()?;

        let token = builder
            .get_user_token(&reqwest::Client::default(), &state, &code)
            .await
            .with_context(|| "Could not get user token")?;

        account.user_access_token = Secret::new(token.access_token.secret().to_owned());

        if let Some(refresh_token) = token.refresh_token {
            account.refresh_token = Secret::new(refresh_token.secret().to_owned());
        }

        account.save()?;

        Ok(account)
    }
}

fn load_account() -> Result<Account> {
    let data: String = read_to_string(ACCOUNT_FILE)?;

    let account: Account = serde_json::from_str(data.as_str())?;

    Ok(account)
}

async fn check_token(account: &mut Account) -> Result<()> {
    if UserToken::from_existing(
        &reqwest::Client::default(),
        AccessToken::new(account.user_access_token.expose_value().to_owned()),
        Some(RefreshToken::new(
            account.refresh_token.expose_value().to_owned(),
        )),
        Some(ClientSecret::new(
            account.client_secret.expose_value().to_owned(),
        )),
    )
    .await
    .is_err()
    {
        let mut user_token = UserToken::from_existing_unchecked(
            AccessToken::new(account.user_access_token.expose_value().to_owned()),
            RefreshToken::new(account.refresh_token.expose_value().to_owned()),
            account.client_id.expose_value().to_owned(),
            ClientSecret::new(account.client_secret.expose_value().to_owned()),
            account.username.clone().into(),
            account.user_id.clone().into(),
            None,
            None,
        );

        user_token
            .refresh_token(&reqwest::Client::default())
            .await?;

        account.user_access_token = Secret::new(user_token.access_token.secret().to_owned());
        account.refresh_token = match user_token.refresh_token {
            Some(token) => Secret::new(token.secret().to_owned()),
            None => Secret::new(account.refresh_token.expose_value().to_owned()),
        };
        account.save()?;
    }

    Ok(())
}
