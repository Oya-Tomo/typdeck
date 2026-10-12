//! Downloads Typst packages over HTTPS.

use std::any::Any;
use std::io::{self, Read};

use typst_kit::downloader::Downloader;

/// A [`Downloader`] with a pure Rust TLS stack, so that building typdeck does
/// not depend on a system OpenSSL.
pub struct HttpDownloader {
    agent: ureq::Agent,
}

impl HttpDownloader {
    pub fn new() -> Self {
        let agent = ureq::Agent::config_builder()
            .user_agent(concat!("typdeck/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();
        Self { agent }
    }
}

impl Downloader for HttpDownloader {
    fn stream(&self, _key: &dyn Any, url: &str) -> io::Result<(Option<usize>, Box<dyn Read>)> {
        match self.agent.get(url).call() {
            Ok(response) => {
                let size = response
                    .body()
                    .content_length()
                    .and_then(|size| size.try_into().ok());
                Ok((size, Box::new(response.into_body().into_reader())))
            }
            Err(ureq::Error::StatusCode(404)) => Err(io::ErrorKind::NotFound.into()),
            Err(error) => Err(io::Error::other(error)),
        }
    }
}
