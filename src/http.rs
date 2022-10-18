use reqwest::Url;
use std::fmt;
use std::net::SocketAddrV4;

pub struct Peer {
    socket_address: SocketAddrV4,
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

    // http://127.0.0.1:6969/announce?
    //    info_hash=%06q3%AC%E5%DD%0CP%27%B9%9D%E5%D4%BAQ%28%28%20%8D%5B
    //    &peer_id=-AZ5760-Suuay04hIXmq
    //    &supportcrypto=1
    //    &port=6881
    //    &azudp=6881
    //    &uploaded=0
    //    &downloaded=0
    //    &left=0
    //    &corrupt=0&event=started&numwant=60&no_peer_id=1&compact=1&key=gWM1LaJa&azver=3
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

pub enum TrackerResponse {
    Failure { message: String },
    Success { interval: usize, peers: Vec<Peer> },
}

#[cfg(test)]
mod test {
    use crate::http::{Event, TrackerRequest};

    #[test]
    fn tracker_request() {
        let info_id = [
            0x06, 0x71, 0x33, 0xAC, 0xE5, 0xDD, 0x0C, 0x50, 0x27, 0xB9, 0x9D, 0xE5, 0xD4, 0xBA,
            0x51, 0x28, 0x28, 0x20, 0x8D, 0x5B,
        ];
        let peer_id = [
            0xDE, 0xAD, 0xBE, 0xEF, 0xBA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
            0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAD,
        ];

        let request = TrackerRequest::new(
            info_id,
            peer_id,
            6882,
            0,
            0,
            356639,
            true,
            Some(Event::Started),
        );

        let url = request.into_url("127.0.0.1", 6969);
        assert_eq!(url.as_str(), "http://127.0.0.1:6969/announce?info_hash=%06q3%AC%E5%DD%0CP%27%B9%9D%E5%D4%BAQ%28%28%20%8D%5B&peer_id=%DE%AD%BE%EF%BA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AD&port=6882&uploaded=0&downloaded=0&left=356639&compact=1&event=started");
    }
}
