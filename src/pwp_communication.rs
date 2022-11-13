use crate::{
    app::App,
    error::Error,
    http::{Event, TrackerRequest, TrackerResponse},
    pwp::{Bitfield, FromBytes, Handshake},
    tcp::TCPSession,
    torrent::Torrent,
};

use reqwest::Url;
use std::{thread, time::Duration};

mod tracker;

pub use self::tracker::TrackerAddress;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PeerToWireState {
    Idle,
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
    last_state: PeerToWireState,
    tcp_session: Option<TCPSession>,
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

        loop {
            if state_machine.last_state != state_machine.state {
                state_machine.state_transition();
            }

            thread::sleep(Duration::from_millis(50));
        }
    }

    fn new(torrent: Torrent) -> Self {
        BitTorrentStateMachine {
            state: PeerToWireState::SendTrackerRequest,
            tcp_session: None,
            last_state: PeerToWireState::Idle,
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
        let tracker_request = Self::build_tracker_request(&self.torrent);
        let tracker_address = Self::tracker_address(&self.torrent)?;
        let response = Self::send_request(tracker_request, tracker_address)?;

        self.tracker_response = Some(response);
        self.state = PeerToWireState::TrackerRequestSent;

        Ok(())
    }

    pub fn tracker_request_sent(&mut self) -> Result<(), Error> {
        println!("{:?}", self.tracker_response);

        if let Some(tracker_response) = self.tracker_response.as_ref() {
            let info_hash = self.torrent.info_hash().unwrap();
            let peer = tracker_response.peers().unwrap().last().unwrap();
            let handshake = Handshake::new(info_hash, Self::PEER_ID);

            self.tcp_session
                .replace(TCPSession::connect(peer.clone()).unwrap());

            if let Some(tcp_session) = &self.tcp_session {
                tcp_session.send(handshake).unwrap();
            } else {
                panic!("cant connect to peer");
            }
        } else {
            panic!("tracker response not received");
        }

        self.state = PeerToWireState::HandshakeSent;
        Ok(())
    }

    pub fn unconnected_with_peers() -> Result<(), Error> {
        println!("I am in the unconnected with peers state");
        Ok(())
    }

    pub fn handshake_sent(&mut self) -> Result<(), Error> {
        println!("I am in the handshake state");

        if let Some(tcp_session) = &self.tcp_session {
            let mut buffer = vec![0; 128];
            tcp_session.receive(&mut buffer).unwrap();
            let (handshake_response, _) = Handshake::from_bytes(&buffer).unwrap();
            println!("handshake response: {:?}", handshake_response);
        }

        self.state = PeerToWireState::ConnectedWithPeer;
        Ok(())
    }
    pub fn wait_handshake() -> Result<(), Error> {
        println!("I am in the wait handshake state");
        Ok(())
    }

    pub fn connection_with_peer(&mut self) -> Result<(), Error> {
        if let Some(tcp_session) = &self.tcp_session {
            let mut buffer = vec![0; 128];
            tcp_session.receive(&mut buffer).unwrap();
            let (handshake_response, _) = Bitfield::from_bytes(&buffer).unwrap();
            println!("bitfield: {:?}", handshake_response);
        }
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
        println!("from {:?} to {:?}", self.last_state, self.state);
        self.last_state = self.state;

        match self.state {
            PeerToWireState::SendTrackerRequest => self.send_tracker_request().unwrap(),
            PeerToWireState::TrackerRequestSent => self.tracker_request_sent().unwrap(),
            PeerToWireState::HandshakeSent => self.handshake_sent().unwrap(),
            PeerToWireState::ConnectedWithPeer => self.connection_with_peer().unwrap(),
            _ => (),
        }
    }
}
