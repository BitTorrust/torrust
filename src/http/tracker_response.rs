use bendy::decoding::{Decoder, Object};
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str,
};

#[derive(Debug)]
pub struct Peer {
    socket_address: SocketAddrV4,
}

impl Peer {
    pub fn from_bytes(chunk: &[u8]) -> Self {
        let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
        let port = (chunk[4] as u16 * 256) + chunk[5] as u16;

        Peer {
            socket_address: SocketAddrV4::new(ip, port),
        }
    }
}

#[derive(Debug)]
pub struct TrackerResponse {
    failure: Option<String>,
    complete: Option<usize>,
    downloaded: Option<usize>,
    incomplete: Option<usize>,
    interval: Option<usize>,
    peers: Option<Vec<Peer>>,
}

impl TrackerResponse {
    pub fn from_bencode(data: &[u8]) -> Self {
        let mut decoder = Decoder::new(data);
        let object = decoder.next_object().unwrap().unwrap();
        let mut dictionary = object.dictionary_or_else(|_| Err(())).unwrap();

        let mut response = Self {
            failure: None,
            complete: None,
            downloaded: None,
            incomplete: None,
            interval: None,
            peers: None,
        };

        while let Ok(pair) = dictionary.next_pair() {
            match pair {
                Some(pair) => response.parse_pair(pair),
                None => break,
            }
        }

        response
    }

    fn parse_pair(&mut self, pair: (&[u8], Object)) {
        match pair {
            (b"complete", value) => {
                self.complete.replace(Self::parse_integer(value));
            }
            (b"downloaded", value) => {
                self.downloaded.replace(Self::parse_integer(value));
            }
            (b"incomplete", value) => {
                self.incomplete.replace(Self::parse_integer(value));
            }
            (b"interval", value) => {
                self.interval.replace(Self::parse_integer(value));
            }
            (b"peers", value) => {
                self.peers.replace(Self::parse_peers(value));
            }
            (b"failure reason", reason) => {
                let error_message = str::from_utf8(reason.try_into_bytes().unwrap())
                    .unwrap()
                    .to_string();
                self.failure.replace(error_message);
            }
            (key, _) => {
                log::warn!(
                    "unhandled key [{}] on tracker response",
                    str::from_utf8(key).unwrap()
                );
            }
        }
    }

    fn parse_integer(object: Object) -> usize {
        object.try_into_integer().unwrap().parse().unwrap()
    }

    fn parse_peers(object: Object) -> Vec<Peer> {
        object
            .try_into_bytes()
            .unwrap()
            .chunks(6)
            .map(|chunk| Peer::from_bytes(chunk))
            .collect()
    }
}
