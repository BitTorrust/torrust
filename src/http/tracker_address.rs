use crate::{error::Error, torrent::Torrent};
use reqwest::Url;

pub struct TrackerAddress {
    host: String,
    port: u16,
}

impl TrackerAddress {
    pub fn from_torrent(torrent: &Torrent) -> Result<Self, Error> {
        let tracker_url = Url::parse(torrent.announce()).map_err(|_| Error::InvalidURLAddress)?;

        TrackerAddress::from_url(tracker_url)
    }

    pub fn from_url(url: Url) -> Result<Self, Error> {
        let host = url.host().ok_or(Error::TrackerHostNotProvided)?.to_string();
        let port = url.port().or_else(|| Some(80)).unwrap();

        Ok(Self { host, port })
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
