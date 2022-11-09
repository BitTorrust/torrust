use crate::error::Error;
use reqwest::Url;

pub struct TrackerAddress {
    host: String,
    port: u16,
}

impl TrackerAddress {
    pub fn from_url(url: Url) -> Result<Self, Error> {
        let host = url.host().ok_or(Error::TrackerHostNotProvided)?.to_string();
        let port = url.port().ok_or(Error::TrackerPortNotProvided)?;

        Ok(Self { host, port })
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
