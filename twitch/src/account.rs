use crate::secret::{Expose, Secret};
use open;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{read_to_string, write},
    io::{prelude::*, BufReader, Error, ErrorKind, Result},
    net::TcpListener,
};
use twitch_api2::twitch_oauth2::{
    refresh_token, AccessToken, ClientId, ClientSecret, RefreshToken, UserToken,
};
use url::Url;

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
        let mut account: Self = load_account()?;

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

    pub async fn new(client_id: String, client_secret: String) -> Result<Self> {
        let mut account = load_account()?;

        account.client_id = Secret::new(client_id);
        account.client_secret = Secret::new(client_secret);

        // TODO allow port to be configured
        let url = match Url::parse("http://localhost:3000/") {
            Ok(parsed_url) => parsed_url,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Could not parse callback URL while getting new token",
                ));
            }
        };

        let mut builder = UserToken::builder(
            ClientId::new(account.client_id.expose_value().to_owned()),
            ClientSecret::new(account.client_secret.expose_value().to_owned()),
            url,
        );

        let (auth_url, _csrf_token) = builder.generate_url();

        let listener = TcpListener::bind("127.0.0.1:3000")?;

        open::that(auth_url.as_str())?;

        for incoming_stream in listener.incoming() {
            match incoming_stream {
                Ok(mut stream) => {
                    let buf_reader = BufReader::new(&mut stream);
                    let http_request: Vec<_> = buf_reader
                        .lines()
                        .filter_map(Result::ok)
                        .take_while(|line| !line.is_empty())
                        .collect();

                    // println!("{:?}", &http_request);

                    match http_request.get(0) {
                        Some(req) => {
                            let query_string = req.split_whitespace().nth(1);
                            let query = match query_string {
                                Some(q) => q,
                                None => {
                                    return Err(Error::new(
                                        ErrorKind::Other,
                                        "Could not get query params from request",
                                    ));
                                }
                            };

                            let response = "HTTP/1.1 200 OK\r\n\r\n";

                            stream.write_all(response.as_bytes())?;
                            stream.flush()?;

                            let parsed = match Url::parse(
                                format!("http://hacky.thing.com{}", &query).as_str(),
                            ) {
                                Ok(result) => result,
                                Err(e) => {
                                    println!("{}", e);
                                    return Err(Error::new(
                                        ErrorKind::Other,
                                        "Could not parse request params",
                                    ));
                                }
                            };

                            let pairs: HashMap<_, _> = parsed.query_pairs().into_owned().collect();

                            // TODO check if has error

                            if let Some(code) = pairs.get("code") {
                                if let Some(state) = pairs.get("state") {
                                    match builder
                                        .get_user_token(&reqwest::Client::default(), state, code)
                                        .await
                                    {
                                        Ok(token) => {
                                            account.user_access_token =
                                                Secret::new(token.access_token.into_string());

                                            if let Some(refresh_token) = token.refresh_token {
                                                account.refresh_token =
                                                    Secret::new(refresh_token.into_string());
                                            }

                                            break;
                                        }
                                        Err(_) => {
                                            return Err(Error::new(
                                                ErrorKind::Other,
                                                "Could not get user token",
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        None => {
                            let response = "HTTP/1.1 400 Bad Request\r\n\r\n400 - Bad Request";

                            stream.write_all(response.as_bytes())?;
                            stream.flush()?;

                            return Err(Error::new(ErrorKind::Other, "Bad request"));
                        }
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // check_token(&mut account).await?;

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
            #[allow(clippy::single_match_else)]
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
