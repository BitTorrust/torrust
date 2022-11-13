use crate::Error;

use {
    crate::http::Peer,
    bendy::decoding::{Decoder, Object},
    std::str,
};

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
    pub fn peers(&self) -> Option<&Vec<Peer>> {
        self.peers.as_ref()
    }

    pub fn from_bencode(data: &[u8]) -> Result<Self, Error> {
        let mut decoder = Decoder::new(data);
        let object = decoder
            .next_object()
            .map_err(|_| Error::UnexpectedResponseFromTracker)?
            .ok_or(Error::UnexpectedResponseFromTracker)?;

        let mut dictionary =
            object.dictionary_or_else(|_| Err(Error::UnexpectedResponseFromTracker))?;
        let mut response = Self::empty();

        while let Ok(pair) = dictionary.next_pair() {
            match pair {
                Some(pair) => response.parse_pair(pair)?,
                None => break,
            }
        }

        Ok(response)
    }

    fn empty() -> Self {
        Self {
            failure: None,
            complete: None,
            downloaded: None,
            incomplete: None,
            interval: None,
            peers: None,
        }
    }

    fn parse_pair(&mut self, pair: (&[u8], Object)) -> Result<(), Error> {
        match pair {
            (b"complete", value) => {
                self.complete.replace(Self::parse_integer(value)?);
            }
            (b"downloaded", value) => {
                self.downloaded.replace(Self::parse_integer(value)?);
            }
            (b"incomplete", value) => {
                self.incomplete.replace(Self::parse_integer(value)?);
            }
            (b"interval", value) => {
                self.interval.replace(Self::parse_integer(value)?);
            }
            (b"peers", value) => {
                self.peers.replace(Self::parse_peers(value)?);
            }
            (b"failure reason", value) => {
                self.failure.replace(Self::parse_failure(value)?);
            }
            (key, _) => {
                log::warn!("unhandled parameter [{}]", str::from_utf8(key).unwrap());
            }
        }

        Ok(())
    }

    fn parse_integer(object: Object) -> Result<usize, Error> {
        let integer = object
            .try_into_integer()
            .map_err(|_| Error::BencodeObjectHasUnexpectedType)?
            .parse()
            .unwrap();

        Ok(integer)
    }

    fn parse_peers(object: Object) -> Result<Vec<Peer>, Error> {
        let peers = object
            .try_into_bytes()
            .map_err(|_| Error::BencodeObjectHasUnexpectedType)?
            .chunks(6)
            .map(|chunk| Peer::from_bytes(chunk))
            .collect();

        Ok(peers)
    }

    fn parse_failure(object: Object) -> Result<String, Error> {
        let failure_message = str::from_utf8(
            object
                .try_into_bytes()
                .map_err(|_| Error::BencodeObjectHasUnexpectedType)?,
        )
        .map_err(|_| Error::TrackerFailureMessageContainsNonUtf8Characters)?
        .to_string();

        Ok(failure_message)
    }
}
