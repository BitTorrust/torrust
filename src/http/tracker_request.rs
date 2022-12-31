use crate::{
    http::{Event, TrackerAddress, TrackerResponse},
    state_machine::StateMachine,
    torrent::Torrent,
    Error,
};

use reqwest::Url;

#[derive(Debug)]
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

    pub fn send_request(
        tracker_request: TrackerRequest,
        tracker: TrackerAddress,
    ) -> Result<TrackerResponse, Error> {
        let url = tracker_request.into_url(tracker.host(), tracker.port())?;

        let mut response =
            reqwest::blocking::get(url).map_err(|_| Error::TrackerConnectionNotPossible)?;
        let mut bencode = Vec::new();
        response.copy_to(&mut bencode).unwrap();
        let parsed_response = TrackerResponse::from_bencode(&bencode);

        parsed_response
    }

    pub fn from_torrent(
        torrent: &Torrent,
        peer_id: [u8; 20],
        left_to_download: u32,
    ) -> TrackerRequest {
        let info_hash = torrent.info_hash();
        let tracker_request = TrackerRequest::new(
            info_hash,
            peer_id,
            StateMachine::CLIENT_PORT,
            0,
            0,
            left_to_download as usize,
            true,
            Some(Event::Started),
        );

        tracker_request
    }
    pub fn into_url(self, host: &str, port: u16) -> Result<Url, Error> {
        let mut url = Url::parse(&format!(
            "http://{}:{}/announce?info_hash={}&peer_id={}",
            host,
            port,
            urlencoding::encode_binary(&self.info_hash),
            urlencoding::encode_binary(&self.peer_id),
        ))
        .map_err(|_| Error::FailedToParseUrl)?;

        self.append_parameters(&mut url);

        Ok(url)
    }

    fn append_parameters(&self, url: &mut Url) {
        url.query_pairs_mut()
            .append_pair("port", &format!("{}", self.port))
            .append_pair("uploaded", &format!("{}", self.uploaded))
            .append_pair("downloaded", &format!("{}", self.downloaded))
            .append_pair("left", &format!("{}", self.left))
            .append_pair("compact", if self.compact { "1" } else { "0" });

        if let Some(event) = &self.event {
            url.query_pairs_mut()
                .append_pair("event", &event.to_string());
        }
    }
}
