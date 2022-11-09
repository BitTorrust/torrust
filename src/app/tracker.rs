use reqwest::Url;

pub struct TrackerAddress {
    host: String,
    port: u16,
}

impl TrackerAddress {
    pub fn from_url(url: Url) -> Self {
        let host = url.host().unwrap().to_string();
        let port = url.port().unwrap();

        Self {
            host: host,
            port: port,
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
