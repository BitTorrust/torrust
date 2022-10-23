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
        let url = format!("http://{host}:{port}/announce?info_hash={info_hash}&peer_id={peer_id}&port={client_port}&uploaded={uploaded}&downloaded={downloaded}&left={left}&compact={compact}",
        host = host,
        port = port,
        info_hash = urlencoding::encode_binary(&self.info_hash),
        peer_id = urlencoding::encode_binary(&self.peer_id),
        client_port= self.port,
        uploaded = self.uploaded,
        downloaded = self.downloaded,
        left = self.left,
        compact = if self.compact { "1" } else { "0 "});

        let url_str = if let Some(event) = self.event {
            format!("{}&event={}", url, event)
        } else {
            url
        };

        Url::parse(&url_str).unwrap()
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
