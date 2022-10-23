use reqwest::Url;
use std::fmt;

pub struct TrackerRequest {
    info_hash: [u8; 20],
    peer_id: [u8; 20],
    port: u16,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    compact: bool,
    event: Option<Event>,
}

impl TrackerRequest {
    pub fn new(
        info_hash: [u8; 20],
        peer_id: [u8; 20],
        port: u16,
        uploaded: usize,
        downloaded: usize,
        left: usize,
        compact: bool,
        event: Option<Event>,
    ) -> Self {
        Self {
            info_hash,
            peer_id,
            port,
            uploaded,
            downloaded,
            left,
            compact,
            event,
        }
    }

    pub fn into_url(self, host: &str, port: u16) -> Url {
        // TODO: stop wrapping here.
        let mut url = Url::parse(&format!(
            "http://{}:{}/announce?info_hash={}&peer_id={}",
            host,
            port,
            urlencoding::encode_binary(&self.info_hash),
            urlencoding::encode_binary(&self.peer_id),
        ))
        .unwrap();

        url.query_pairs_mut()
            .append_pair("port", &format!("{}", self.port))
            .append_pair("uploaded", &format!("{}", self.uploaded))
            .append_pair("downloaded", &format!("{}", self.downloaded))
            .append_pair("left", &format!("{}", self.left))
            .append_pair("compact", if self.compact { "1" } else { "0" });

        if let Some(event) = self.event {
            url.query_pairs_mut()
                .append_pair("event", &event.to_string());
        }

        url
    }
}

pub enum Event {
    Started,
    Stopped,
    Completed,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Event::Started => "started",
            Event::Stopped => "stopped",
            Event::Completed => "completed",
        };

        write!(f, "{}", text)
    }
}
