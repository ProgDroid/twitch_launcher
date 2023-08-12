use std::{
    collections::HashMap,
    io::{BufRead, BufReader, ErrorKind, Write},
    net::{TcpListener, TcpStream},
};

use anyhow::{Context, Error, Result};
use url::Url;

#[must_use]
pub struct Server {
    port: u16,
    listener: TcpListener,
}

impl Server {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(port: u16) -> Result<Self> {
        Ok(Self {
            port,
            listener: bind_listener(port)?,
        })
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn redirect_url(&self) -> Result<Url> {
        Url::parse(format!("http://localhost:{}/", &self.port).as_str()).with_context(|| {
            format!(
                "Could not parse redirect URL with given port {}",
                &self.port
            )
        })
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn get_callback_data(&self) -> Result<(String, String)> {
        if let Some(mut stream) = self.listener.incoming().flatten().next() {
            let buf_reader = BufReader::new(&mut stream);
            let http_request: Vec<_> = buf_reader
                .lines()
                .filter_map(Result::ok)
                .take_while(|line| !line.is_empty())
                .collect();

            if let Some(req) = http_request.get(0) {
                respond_success(stream)?;

                let query = req
                    .split_whitespace()
                    .nth(1)
                    .with_context(|| "Could not get query params from request")?;

                let pairs = get_params(query)?;

                if let Some(error_description) = pairs.get("error_description") {
                    return Err(Error::new(std::io::Error::new(
                        ErrorKind::Other,
                        format!("Got error from Twitch - {error_description}",),
                    )));
                }

                if let Some(code) = pairs.get("code") {
                    if let Some(state) = pairs.get("state") {
                        return Ok((code.clone(), state.clone()));
                    }
                }

                return Err(Error::new(std::io::Error::new(
                    ErrorKind::Other,
                    "Code and state string not in query params",
                )));
            }

            respond_failure(stream)?;

            return Err(Error::new(std::io::Error::new(
                ErrorKind::Other,
                "Bad request",
            )));
        }

        Err(Error::new(std::io::Error::new(
            ErrorKind::Other,
            "No connection made",
        )))
    }
}

fn bind_listener(port: u16) -> Result<TcpListener> {
    TcpListener::bind(format!("127.0.0.1:{}", &port))
        .with_context(|| format!("Could not bind listener to port {port}"))
}

fn respond_success(stream: TcpStream) -> Result<()> {
    respond("HTTP/1.1 200 OK\r\n\r\n", stream)
}

fn respond_failure(stream: TcpStream) -> Result<()> {
    respond("HTTP/1.1 400 Bad Request\r\n\r\n400 - Bad Request", stream)
}

fn respond(response: &str, mut stream: TcpStream) -> Result<()> {
    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

fn get_params(query: &str) -> Result<HashMap<String, String>> {
    let parsed = Url::parse(format!("http://hacky.thing.com{}", &query).as_str())
        .with_context(|| "Could not parse request params")?;

    Ok(parsed.query_pairs().into_owned().collect())
}
