use crate::{
    secret::{ExposeSecret, Secret},
    twitch_account::TwitchAccount,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::read_to_string,
    io::Result,
    process::{Command, Output},
};
use tokio;
use tokio::sync::mpsc;
use twitch_api2::{
    helix::streams::GetStreamsRequest,
    twitch_oauth2::{client::reqwest_http_client, AccessToken, UserToken},
    HelixClient,
};

const CHANNELS_FILE: &str = "channels.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[repr(usize)]
pub enum ChannelStatus {
    Unknown = 0,
    Online = 1,
    Offline = 2,
}

impl Default for ChannelStatus {
    fn default() -> Self {
        ChannelStatus::Unknown
    }
}

impl ChannelStatus {
    pub fn message(self: &Self) -> &str {
        match self {
            ChannelStatus::Unknown => "...  ",
            ChannelStatus::Online => "online",
            ChannelStatus::Offline => "offline",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Channel {
    pub friendly_name: String,
    pub handle: String,
    #[serde(default = "ChannelStatus::default")]
    pub status: ChannelStatus,
}

impl Channel {
    pub async fn update_status(handle: String, user_access_token: Secret) -> ChannelStatus {
        let token = AccessToken::new(user_access_token.expose_value().to_string());

        let token = match UserToken::from_existing(reqwest_http_client, token, None, None).await {
            Ok(token) => token,
            Err(_) => {
                return ChannelStatus::Offline;
            }
        };

        let client: HelixClient<reqwest::Client> = HelixClient::new();

        let req = GetStreamsRequest::builder()
            .user_login(vec![handle])
            .build();

        let response = client.req_get(req, &token).await.unwrap();

        return if response.data.is_empty() {
            ChannelStatus::Offline
        } else {
            ChannelStatus::Online
        };
    }

    // TODO popup if channel is offline (are you sure?)
    pub fn launch(self: &Self) -> Result<Output> {
        Command::new("powershell")
            .arg("Start-Process")
            .arg("streamlink")
            .arg(format!("twitch.tv/{}", self.handle))
            .arg("-WindowStyle")
            .arg("Hidden")
            .output()
    }

    pub fn launch_chat(self: &Self) -> Result<Output> {
        Command::new("powershell")
            .arg("Start-Process")
            .arg("\"C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\Chatterino\"")
            .arg(format!("\"-c {}\"", self.handle))
            .output()
    }
}

// TODO consider returning result here? handle not being able to deserialise JSON
pub fn load_channels(
    twitch_account: &TwitchAccount,
) -> Result<(Vec<Channel>, mpsc::Receiver<(String, ChannelStatus)>)> {
    let data: String = read_to_string(CHANNELS_FILE)?;

    let channels: Vec<Channel> = serde_json::from_str(data.as_str())?;

    let (sender, receiver) = mpsc::channel(channels.len());

    // TODO add cache of statuses?
    for channel in &channels {
        let tx = sender.clone();

        let channel_handle: String = String::from(channel.handle.as_str());
        let secret: Secret =
            Secret::new(twitch_account.user_access_token.expose_value().to_string());

        tokio::spawn(async move {
            tx.send((
                String::from(&channel_handle),
                Channel::update_status(channel_handle, secret).await,
            ))
            .await
        });
    }

    Ok((channels, receiver))
}

// TODO need to add account configuration
// TODO github actions to check code?
// TODO upgrade twitch api crate
