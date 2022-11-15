use crate::{
    error::Error,
    file_management::BlockReaderWriter,
    http::{Event, TrackerRequest, TrackerResponse},
    pwp::{
        Bitfield, FromBytes, Handshake, Have, Interested, NotInterested, Piece, Request, Unchoke,
    },
    tcp::TCPSession,
    torrent,
    torrent::Torrent,
};

use bit_vec::BitVec;
use reqwest::Url;
use std::{path::PathBuf, thread, time::Duration};

mod tracker_address;
pub use tracker_address::TrackerAddress;

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

    Done,
}

pub struct BitTorrentStateMachine {
    last_state: PeerToWireState,
    tcp_session: Option<TCPSession>,
    state: PeerToWireState,
    torrent: Torrent,
    tracker_response: Option<TrackerResponse>,
    peer_bitfield: Option<Bitfield>,
    working_directory: PathBuf,
}

impl BitTorrentStateMachine {
    const PEER_ID: [u8; 20] = [
        0xDE, 0xAD, 0xBE, 0xEF, 0xBA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
        0xAA, 0xAA, 0xAA, 0xAA, 0xAD,
    ];

    pub fn run(torrent: Torrent, working_directory: &PathBuf) {
        let mut state_machine = BitTorrentStateMachine::new(torrent, working_directory);

        loop {
            if state_machine.state == PeerToWireState::Done {
                state_machine.done().unwrap();
                break;
            }

            if state_machine.last_state != state_machine.state {
                state_machine.state_transition();
            }

            thread::sleep(Duration::from_millis(50));
        }
    }

    fn new(torrent: Torrent, working_directory: &PathBuf) -> Self {
        BitTorrentStateMachine {
            state: PeerToWireState::SendTrackerRequest,
            tcp_session: None,
            last_state: PeerToWireState::Idle,
            torrent,
            tracker_response: None,
            peer_bitfield: None,
            working_directory: working_directory.to_owned(),
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

        let tracker_response = self.tracker_response()?;
        let info_hash = self.torrent.info_hash().unwrap();
        let handshake = Handshake::new(info_hash, Self::PEER_ID);

        // TODO: talk to the right peers
        let peer = tracker_response.peers().unwrap().last().unwrap();
        let tcp_session = TCPSession::connect(peer.clone()).unwrap();
        self.tcp_session.replace(tcp_session);

        let tcp_session = self.tcp_session()?;
        tcp_session.send(handshake).unwrap();

        self.state = PeerToWireState::HandshakeSent;
        Ok(())
    }

    pub fn unconnected_with_peers() -> Result<(), Error> {
        unimplemented!()
    }

    pub fn handshake_sent(&mut self) -> Result<(), Error> {
        let tcp_session = self.tcp_session()?;
        let mut buffer = vec![0; 68];
        tcp_session.receive(&mut buffer).unwrap();
        let (handshake_response, _) = Handshake::from_bytes(&buffer).unwrap();
        println!("{:?}", handshake_response);

        self.state = PeerToWireState::NotInterestedAndChoked;
        Ok(())
    }

    pub fn wait_handshake() -> Result<(), Error> {
        unimplemented!()
    }

    pub fn not_interested_and_choked(&mut self) -> Result<(), Error> {
        let received_bitfield = {
            let tcp_session = self.tcp_session()?;
            let bitfield_length =
                5 + torrent::div_ceil(self.torrent.number_of_pieces().unwrap(), 8);

            let mut buffer = vec![0; bitfield_length as usize];
            tcp_session.receive(&mut buffer).unwrap();

            let (received_bitfield, _) = Bitfield::from_bytes(&buffer).unwrap();
            println!("{:?}", received_bitfield);

            let bitfield = Bitfield::new(BitVec::from_elem(
                self.torrent.number_of_pieces().unwrap() as usize,
                false,
            ));
            tcp_session.send(bitfield).unwrap();

            let interested = Interested::new();
            tcp_session.send(interested).unwrap();

            received_bitfield
        };

        self.peer_bitfield.replace(received_bitfield);
        self.state = PeerToWireState::InterestedAndChoked;

        Ok(())
    }

    pub fn interested_and_choked(&mut self) -> Result<(), Error> {
        let tcp_session = self.tcp_session()?;
        let mut buffer = vec![0; 5];
        tcp_session.receive(&mut buffer).unwrap();

        let unchoke = Unchoke::from_bytes(&buffer).unwrap();
        println!("{:?}", unchoke);

        self.download_pieces(&tcp_session);

        self.state = PeerToWireState::Done;

        Ok(())
    }

    fn download_pieces(&self, tcp_session: &TCPSession) {
        println!("Downloading pieces");

        let total_length = self.torrent.total_length_in_bytes().unwrap();
        let piece_length = self.torrent.piece_length_in_bytes().unwrap();

        let block_size = 16 * 1024;
        let total_blocks = torrent::div_ceil(total_length, block_size);
        let blocks_per_piece = piece_length / block_size;

        let filename = self.filepath();
        println!("Saving file to {:?}", filename);
        let file_on_disk =
            BlockReaderWriter::new(&filename, piece_length, total_length as usize).unwrap();

        for block in 0..total_blocks {
            let bytes_to_read = 13
                + if block == total_blocks - 1 {
                    total_length % block_size
                } else {
                    block_size
                } as usize;

            let piece_index = block / blocks_per_piece;
            let block_offset = (block % blocks_per_piece) * block_size;

            let request = Request::new(piece_index, block_offset, bytes_to_read as u32 - 13);
            tcp_session.send(request).unwrap();

            std::thread::sleep(std::time::Duration::from_millis(20));
            let mut buffer = vec![0; bytes_to_read];

            tcp_session.receive(&mut buffer).unwrap();
            let (piece, _) = Piece::from_bytes(&buffer).unwrap();

            file_on_disk
                .write(piece_index, block_offset, piece.data())
                .unwrap();

            if block_offset == block_size * (blocks_per_piece - 1) {
                let have = Have::new(piece_index);
                tcp_session.send(have).unwrap();
            }
        }

        let not_interested = NotInterested::new();
        tcp_session.send(not_interested).unwrap();
    }

    pub fn interested_and_unchoked(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn not_interested_and_unchoked() -> Result<(), Error> {
        unimplemented!()
    }

    pub fn not_interesting_and_choking() -> Result<(), Error> {
        unimplemented!()
    }

    pub fn interesting_and_choking() -> Result<(), Error> {
        unimplemented!()
    }

    pub fn interesting_and_unchoking() -> Result<(), Error> {
        unimplemented!()
    }

    pub fn not_interesting_and_unchoking() -> Result<(), Error> {
        unimplemented!()
    }

    pub fn done(&self) -> Result<(), Error> {
        Ok(())
    }

    pub fn state_transition(&mut self) {
        println!("From state {:?} to {:?}", self.last_state, self.state);
        self.last_state = self.state;

        match self.state {
            PeerToWireState::SendTrackerRequest => self.send_tracker_request(),
            PeerToWireState::TrackerRequestSent => self.tracker_request_sent(),
            PeerToWireState::HandshakeSent => self.handshake_sent(),
            PeerToWireState::NotInterestedAndChoked => self.not_interested_and_choked(),
            PeerToWireState::InterestedAndChoked => self.interested_and_choked(),
            PeerToWireState::InterestedAndUnchoked => self.interested_and_unchoked(),
            PeerToWireState::Done => self.done(),
            _ => unimplemented!(),
        }
        .unwrap();
    }

    fn tcp_session(&self) -> Result<&TCPSession, Error> {
        self.tcp_session
            .as_ref()
            .ok_or(Error::TcpSessionDoesNotExist)
    }

    fn tracker_response(&self) -> Result<&TrackerResponse, Error> {
        self.tracker_response
            .as_ref()
            .ok_or(Error::TrackerConnectionNotPossible)
    }

    fn filepath(&self) -> PathBuf {
        let complete_path = self.working_directory.join(self.torrent.name().unwrap());

        complete_path
    }
}
