use crate::{
    app::App,
    error::Error,
    http::{Event, TrackerRequest, TrackerResponse},
    torrent::Torrent,
};

use reqwest::Url;

mod tracker;

pub use self::tracker::TrackerAddress;

pub enum PeerToWireState {
    SendTrackerRequest,
    TrackerRequestSent,
    UnconnectedWithPeers,
    HandshakeSent,
    WaitHandshake,
    ConnectedWithPeer,
    ConnectionClosed,
    //Download states
    NotInterestedAndChoked,
    InterestedAndChoked,
    InterestedAndUnchoked,
    NotInterestedAndUnchoked,
    //upload states, ahora se puede adjuntar aqui porque sí son excluyentes, no lo serán en el caso de multiples clientes. En este descargo o envío
    // para el caso diferente de 0% y 100%, puedo tener estados de descarga y estados de carga con un mismo par, entonces habría que hacer 2 maquinas de estados
    NotInterestingAndChoking,
    InterestingAndChoking,
    InterestingAndUnchoking,
    NotInterestingAndUnchoking,
}
pub struct BitTorrentStateMachine {
    state: PeerToWireState,
    torrent: Torrent,
    tracker_response: Option<TrackerResponse>,
}

impl BitTorrentStateMachine {
    const PEER_ID: [u8; 20] = [
        0xDE, 0xAD, 0xBE, 0xEF, 0xBA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
        0xAA, 0xAA, 0xAA, 0xAA, 0xAD,
    ];

    pub fn run(torrent: Torrent) {
        let mut state_machine = BitTorrentStateMachine::new(torrent);
        state_machine.state_transition();
    }

    fn new(torrent: Torrent) -> Self {
        BitTorrentStateMachine {
            state: PeerToWireState::SendTrackerRequest,
            torrent,
            tracker_response: None,
        }
    }

    fn tracker_address(torrent: &Torrent) -> Result<TrackerAddress, Error> {
        let tracker_url =
            Url::parse(torrent.announce().unwrap()).map_err(|_| Error::InvalidURLAddress)?;

        let tracker_address = TrackerAddress::from_url(tracker_url)?;

        Ok(tracker_address)
    }

    fn build_tracker_request(torrent: &Torrent) -> TrackerRequest {
        let info_hash = torrent.info_hash().unwrap();
        let left_to_download = torrent.total_length_in_bytes().unwrap();
        let tracker_request = TrackerRequest::new(
            info_hash,
            Self::PEER_ID,
            6882,
            0,
            0,
            left_to_download as usize,
            true,
            Some(Event::Started),
        );

        tracker_request
    }

    fn send_request(
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

    pub fn send_tracker_request(&mut self) -> Result<(), Error> {
        println!("I am in the send tracker request state");

        let tracker_request = Self::build_tracker_request(&self.torrent);
        let tracker_address = Self::tracker_address(&self.torrent)?;
        let response = Self::send_request(tracker_request, tracker_address)?;

        self.tracker_response = Some(response);
        self.state = PeerToWireState::TrackerRequestSent;
        self.state_transition();
        Ok(())
    }

    pub fn tracker_request_sent(&self) -> Result<(), Error> {
        println!("{:?}", self.tracker_response);

        Ok(())
    }

    pub fn unconnected_with_peers() -> Result<(), Error> {
        println!("I am in the unconnected with peers state");
        Ok(())
    }

    pub fn handshake_sent() -> Result<(), Error> {
        println!("I am in the handshake state");
        Ok(())
    }
    pub fn wait_handshake() -> Result<(), Error> {
        println!("I am in the wait handshake state");
        Ok(())
    }

    pub fn connection_with_peer() -> Result<(), Error> {
        println!("I am in the connection with peer state");
        Ok(())
    }
    pub fn not_interested_and_choked() -> Result<(), Error> {
        println!("I am in the Not interested and choked state");
        Ok(())
    }

    pub fn interested_and_choked() -> Result<(), Error> {
        println!("I am in the interested and choked state");
        Ok(())
    }

    pub fn interested_and_unchoked() -> Result<(), Error> {
        println!("I am in the interested and unchoked state");
        Ok(())
    }

    pub fn not_interested_and_unchoked() -> Result<(), Error> {
        println!("I am in the not interested and unchoked state");
        Ok(())
    }

    pub fn not_interesting_and_choking() -> Result<(), Error> {
        println!("I am in the not interesting and choking state");
        Ok(())
    }

    pub fn interesting_and_choking() -> Result<(), Error> {
        println!("I am in the interesting and choking state");
        Ok(())
    }

    pub fn interesting_and_unchoking() -> Result<(), Error> {
        println!("I am in the interesting and unchoking state");
        Ok(())
    }

    pub fn not_interesting_and_unchoking() -> Result<(), Error> {
        println!("I am in the not interesting and unchoking state");
        Ok(())
    }

    pub fn state_transition(&mut self) {
        match &self.state {
            SendTrackerRequest => self.send_tracker_request(),
            TrackerRequestSent => self.tracker_request_sent(),
            UnconnectedWithPeers => Self::unconnected_with_peers(),
            HandshakeSent => Self::handshake_sent(),

            NotInterestedAndChoked => Self::not_interested_and_choked(),
            InterestedAndChoked => Self::interested_and_choked(),
            InterestedAndUnchoked => Self::interested_and_unchoked(),
            NotInterestedAndUnchoked => Self::not_interested_and_unchoked(),
        }
        .unwrap();
    }
}
