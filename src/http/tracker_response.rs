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
pub enum TrackerResponse {
    Failure {
        message: String,
    },
    Success {
        complete: usize,
        downloaded: usize,
        incomplete: usize,
        interval: usize,
        peers: Vec<Peer>,
    },
}

impl TrackerResponse {
    pub fn from_bencode(data: &[u8]) -> Self {
        println!("bencode: {}", unsafe { str::from_utf8_unchecked(data) });

        let mut decoder = Decoder::new(data);
        let object = decoder.next_object().unwrap().unwrap();
        let mut dictionary = object.dictionary_or_else(|_| Err(())).unwrap();

        let mut complete = None;
        let mut downloaded = None;
        let mut incomplete = None;
        let mut interval = None;
        let mut peers = None;
        let mut failure_reason = None;

        while let Ok(pair) = dictionary.next_pair() {
            match pair {
                Some((b"complete", value)) => {
                    complete.replace(Self::parse_integer(value));
                }
                Some((b"downloaded", value)) => {
                    downloaded.replace(Self::parse_integer(value));
                }
                Some((b"incomplete", value)) => {
                    incomplete.replace(Self::parse_integer(value));
                }
                Some((b"interval", value)) => {
                    interval.replace(Self::parse_integer(value));
                }
                Some((b"peers", value)) => {
                    peers.replace(Self::parse_peers(value));
                }
                Some((b"failure reason", reason)) => {
                    let error_message = str::from_utf8(reason.try_into_bytes().unwrap())
                        .unwrap()
                        .to_string();
                    failure_reason.replace(error_message);
                }
                Some((key, _)) => {
                    log::warn!(
                        "unhandled key [{}] on tracker response",
                        str::from_utf8(key).unwrap()
                    );
                }
                None => break,
            }
        }

        if let Some(message) = failure_reason {
            TrackerResponse::Failure { message }
        } else {
            TrackerResponse::Success {
                // TODO: stop unwrapping here, some fields may not be available
                complete: complete.unwrap(),
                downloaded: downloaded.unwrap(),
                incomplete: incomplete.unwrap(),
                interval: interval.unwrap(),
                peers: peers.unwrap(),
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
