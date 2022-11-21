use crate::{
    error::Error,
    file_management::BlockReaderWriter,
    http::{Event, TrackerRequest, TrackerResponse},
    pwp::{
        from_bytes, Bitfield, FromBytes, Handshake, Have, Interested, Message, MessageType,
        NotInterested, Piece, Request, Unchoke,
    },
    tcp::TCPSessionMock,
    torrent,
    torrent::Torrent,
};

use bit_vec::BitVec;
use reqwest::Url;
use std::{net::TcpListener, path::PathBuf, thread, time::Duration};

mod tracker_address;
pub use tracker_address::TrackerAddress;

const PEER_ID: [u8; 20] = [
    0xDE, 0xAD, 0xBE, 0xEF, 0xBA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
    0xAA, 0xAA, 0xAA, 0xAD,
];

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
    NotInterestingAndChoking,
    InterestingAndChoking,
    InterestingAndUnchoking,
    NotInterestingAndUnchoking,

    Done,
}
#[derive(PartialEq)]
pub enum ClientMode {
    Leecher,
    Seeder,
}

// Client : my application
// Peer : the client I am communicating with
pub struct BitTorrentStateMachine {
    last_state: PeerToWireState,
    tcp_session: Option<TCPSessionMock>,
    state: PeerToWireState,
    torrent: Torrent,
    tracker_response: Option<TrackerResponse>,
    peer_bitfield: Option<Bitfield>,
    working_directory: PathBuf,
    listener: Option<TcpListener>,
}

impl BitTorrentStateMachine {
    pub fn run(torrent: Torrent, working_directory: &PathBuf) {
        let mut state_machine = BitTorrentStateMachine::new(torrent, working_directory);

        loop {
            if state_machine.state == PeerToWireState::Done {
                state_machine.done().unwrap();
                break;
            }

            state_machine.state_transition();

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
            listener: None,
        }
    }

    fn tracker_address(torrent: &Torrent) -> Result<TrackerAddress, Error> {
        let tracker_url = Url::parse(torrent.announce()).map_err(|_| Error::InvalidURLAddress)?;

        let tracker_address = TrackerAddress::from_url(tracker_url)?;

        Ok(tracker_address)
    }

    pub fn send_tracker_request(&mut self) -> Result<(), Error> {
        let tracker_request = TrackerRequest::from_torrent(&self.torrent, PEER_ID);
        let tracker_address = Self::tracker_address(&self.torrent)?;
        let response = TrackerRequest::send_request(tracker_request, tracker_address)?;

        self.tracker_response = Some(response);
        self.state = PeerToWireState::TrackerRequestSent;

        Ok(())
    }

    pub fn tracker_request_sent(&mut self) -> Result<(), Error> {
        println!("{:?}", self.tracker_response);

        let tracker_response = self.tracker_response()?;
        let info_hash = self.torrent.info_hash();
        let handshake = Handshake::new(info_hash, PEER_ID);
        let peer = tracker_response.peers().unwrap().last().unwrap();

        // TODO: Remove after sprint seeder 100% & leecher 0%
        if self.client_mode() == ClientMode::Leecher {
            // TODO: talk to the right peers
            let tcp_session = TCPSessionMock::connect(peer.clone()).unwrap();
            self.tcp_session.replace(tcp_session);

            let tcp_session = self.tcp_session()?;
            tcp_session.send(handshake).unwrap();

            self.state = PeerToWireState::HandshakeSent;
        } else {
            let listener = TCPSessionMock::listen()?;
            self.listener.replace(listener);
            self.state = PeerToWireState::WaitHandshake;
        }

        Ok(())
    }

    pub fn unconnected_with_peers() -> Result<(), Error> {
        unimplemented!()
    }

    pub fn handshake_sent(&mut self) -> Result<(), Error> {
        let tcp_session = self.tcp_session()?;
        let handshake_message = match tcp_session.receive().unwrap() {
            Some(message) => message,
            None => unimplemented!(),
        };

        let handshake = match handshake_message {
            Message::Handshake(handshake) => handshake,
            _ => panic!("Expected handshake."),
        };

        println!("{:?}", handshake);

        self.state = PeerToWireState::NotInterestedAndChoked;
        Ok(())
    }

    pub fn wait_handshake(&mut self) -> Result<(), Error> {
        let info_hash = self.torrent.info_hash();
        let number_of_pieces = self.torrent.number_of_pieces() as usize;

        {
            let listener = self.listener()?;
            let tcp_session = TCPSessionMock::accept(
                listener
                    .try_clone()
                    .map_err(|_| Error::FailedToCloneSocketHandle)?,
            )?;
            self.tcp_session.replace(tcp_session);
        }
        let tcp_session = self.tcp_session()?;

        let handshake_message = tcp_session.receive().unwrap().unwrap();
        let handshake = match handshake_message {
            Message::Handshake(message) => message,
            _ => panic!("expected handshake"),
        };
        println!("Handshake received => {:?}", handshake);

        let handshake = Handshake::new(info_hash, PEER_ID);
        tcp_session.send(handshake).unwrap();

        let bitfield = Bitfield::new(BitVec::from_elem(number_of_pieces, true));
        tcp_session.send(bitfield).unwrap();

        self.state = PeerToWireState::NotInterestingAndChoking;

        Ok(())
    }

    pub fn not_interested_and_choked(&mut self) -> Result<(), Error> {
        let total_pieces = self.torrent.number_of_pieces() as usize;

        let received_bitfield = {
            let tcp_session = self.tcp_session()?;
            let bitfield_message = tcp_session.receive().unwrap().unwrap();
            let received_bitfield = match bitfield_message {
                Message::Bitfield(bitfield) => bitfield,
                _ => panic!("expected bitfield"),
            };

            println!("{:?}", received_bitfield);

            let bitfield = Bitfield::new(BitVec::from_elem(total_pieces, false));
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
        let unchoke_message = tcp_session.receive().unwrap().unwrap();

        let unchoke = match unchoke_message {
            Message::Unchoke(unchoke) => unchoke,
            _ => panic!("expected unchoke"),
        };

        println!("{:?}", unchoke);

        self.state = PeerToWireState::InterestedAndUnchoked;

        Ok(())
    }

    pub fn interested_and_unchoked(&mut self) -> Result<(), Error> {
        self.download_pieces();

        self.state = PeerToWireState::Done;

        Ok(())
    }

    pub fn not_interested_and_unchoked() -> Result<(), Error> {
        unimplemented!()
    }

    pub fn not_interesting_and_choking(&mut self) -> Result<(), Error> {
        let tcp_session = self.tcp_session()?;
        let bitfield_message = tcp_session.receive().unwrap().unwrap();
        let bitfield = match bitfield_message {
            Message::Bitfield(bitfield) => bitfield,
            _ => panic!("expected bitfield"),
        };

        println!("Bitfield received: {:?}", bitfield);

        let interested_message = tcp_session.receive().unwrap().unwrap();
        let interested = match interested_message {
            Message::Interested(interested) => interested,
            _ => panic!("expected interested"),
        };

        println!("Interested received: {:?}", interested);
        self.peer_bitfield.replace(bitfield);
        self.state = PeerToWireState::InterestingAndChoking;

        Ok(())
    }

    // TODO: decide when to choke or unchoke a peer
    pub fn interesting_and_choking(&mut self) -> Result<(), Error> {
        let tcp_session = self.tcp_session()?;
        let unchoke_message = Unchoke::new();

        tcp_session.send(unchoke_message).unwrap();
        self.state = PeerToWireState::InterestingAndUnchoking;
        Ok(())
    }

    // TODO: Use receive() method described on tcp_session_mock.rs
    pub fn interesting_and_unchoking(&mut self) -> Result<(), Error> {
        let tcp_session = self.tcp_session()?;

        let received_message = tcp_session.receive()?;
        println!("Received: {:?}", received_message);
        match received_message {
            Some(Message::Request(request)) => {
                self.upload_pieces(request);
                self.state = PeerToWireState::InterestingAndUnchoking;
            }
            Some(Message::Have(_)) => self.state = PeerToWireState::InterestingAndUnchoking,
            Some(Message::NotInterested(_)) => self.state = PeerToWireState::Done,
            _ => unimplemented!(),
        }

        Ok(())
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
            // Leecher states
            PeerToWireState::HandshakeSent => self.handshake_sent(),
            PeerToWireState::NotInterestedAndChoked => self.not_interested_and_choked(),
            PeerToWireState::InterestedAndChoked => self.interested_and_choked(),
            PeerToWireState::InterestedAndUnchoked => self.interested_and_unchoked(),
            // Seeder states
            PeerToWireState::WaitHandshake => self.wait_handshake(),
            PeerToWireState::NotInterestingAndChoking => self.not_interesting_and_choking(),
            PeerToWireState::InterestingAndChoking => self.interesting_and_choking(),
            PeerToWireState::InterestingAndUnchoking => self.interesting_and_unchoking(),

            PeerToWireState::Done => self.done(),
            _ => unimplemented!(),
        }
        .unwrap();
    }

    fn upload_pieces(&mut self, request: Request) {
        println!("Uploading piece");

        let total_length = self.torrent.total_length_in_bytes();
        let piece_length = self.torrent.piece_length_in_bytes();

        let filename = &self.working_directory;
        println!("Uploading from {:?}", filename);
        let file_on_disk =
            BlockReaderWriter::new(filename, piece_length, total_length as usize).unwrap();

        let piece_index = request.piece_index();
        let block_offset = request.begin_offset();
        let data = file_on_disk.read(piece_index, block_offset).unwrap();

        let piece = Piece::new(piece_index, block_offset, data);

        let tcp_session = self.tcp_session().unwrap();
        tcp_session.send(piece).unwrap();
    }

    fn download_pieces(&mut self) {
        println!("Downloading pieces");

        let total_length = self.torrent.total_length_in_bytes();
        let piece_length = self.torrent.piece_length_in_bytes();

        let block_size = 16 * 1024;
        let total_blocks = torrent::div_ceil(total_length, block_size);
        let blocks_per_piece = piece_length / block_size;

        let filename = self.filepath();
        println!("Saving file to {:?}", filename);
        let file_on_disk =
            BlockReaderWriter::new(&filename, piece_length, total_length as usize).unwrap();

        let tcp_session = self.tcp_session().unwrap();
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

            let piece_message = tcp_session.receive().unwrap().unwrap();
            let piece = match piece_message {
                Message::Piece(piece) => piece,
                _ => panic!("expected piece"),
            };

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

    fn tcp_session(&mut self) -> Result<&mut TCPSessionMock, Error> {
        self.tcp_session
            .as_mut()
            .ok_or(Error::TcpSessionDoesNotExist)
    }

    fn listener(&self) -> Result<&TcpListener, Error> {
        self.listener.as_ref().ok_or(Error::TcpListenerDoesNotExist)
    }

    fn tracker_response(&self) -> Result<&TrackerResponse, Error> {
        self.tracker_response
            .as_ref()
            .ok_or(Error::TrackerConnectionNotPossible)
    }

    fn filepath(&self) -> PathBuf {
        let complete_path = self.working_directory.join(self.torrent.name());

        complete_path
    }

    // TODO: erase after sprint-1
    // Replacement: A piece check method
    fn client_mode(&self) -> ClientMode {
        let client_mode;

        if self.working_directory.is_file() {
            client_mode = ClientMode::Seeder;
        } else {
            client_mode = ClientMode::Leecher;
        }

        client_mode
    }
}
