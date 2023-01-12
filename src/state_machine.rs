use {
    crate::{
        error::Error,
        file_management::local_bitfield,
        http::Peer,
        http::{TrackerAddress, TrackerRequest},
        pwp::{
            Bitfield, Handshake, Have, Interested, Message, NotInterested, Piece, Request, Unchoke,
        },
        torrent::{self, Torrent},
    },
    crossbeam_channel::Receiver,
    std::path::PathBuf,
    std::thread,
};

mod tcp_handler;
use std::{collections::HashMap, time::Duration};

use bit_vec::BitVec;
use tcp_handler::TcpHandler;

mod wait;
pub use wait::Wait;

pub(crate) mod identity;
use identity::generate_random_identity;

use crate::{
    http::TrackerResponse,
    pieces_selection::{DistributedSelector, PiecesSelection},
    BlockReaderWriter,
};

#[derive(Debug)]
pub struct StateMachine {
    message_receiver: Receiver<(Peer, Message)>,
    tcp_handler: TcpHandler,
    torrent: Torrent,
    client_id: [u8; 20],
    seeder_peers: HashMap<Peer, MyLeecherState>,
    peers_bitfield: HashMap<Peer, BitVec>,
    leecher_peers: HashMap<Peer, MySeederState>,
    bitfield: BitVec,
    requested_pieces: BitVec,
    block_reader_writer: BlockReaderWriter,
    blocks_by_piece: HashMap<u32, usize>,
    mock_peers: bool,
}

#[derive(Debug, Clone)]
enum MySeederState {
    //Upload states
    WaitingHandshake,
    NotInterestingAndChoking,
    InterestingAndUnchoking,
}

#[derive(Debug, Clone)]
enum MyLeecherState {
    //Download states
    Unconnected,
    WaitingHandshake,
    WaitingBitfield,
    BitfieldSent,
    NotInterestedAndChoked,
    InterestedAndChoked,
    InterestedAndUnchoked,
    NotInterestedAndUnchoked,
}

impl StateMachine {
    pub const CLIENT_PORT: u16 = 6882;

    pub fn new(torrent: Torrent, working_directory: &PathBuf, mock_peers: bool) -> Self {
        let (message_sender, message_receiver) = crossbeam_channel::unbounded();
        let tcp_handler = TcpHandler::new(message_sender);
        let bitfield = local_bitfield(&torrent, working_directory);
        let bitfield_length = bitfield.len();
        let filename;
        if working_directory.is_dir() {
            filename = working_directory.join(torrent.name());
        } else {
            filename = working_directory.to_path_buf();
        }
        let piece_length = torrent.piece_length_in_bytes();
        let file_size = torrent.total_length_in_bytes();
        let block_reader_writer =
            BlockReaderWriter::new(&filename, piece_length, file_size as usize).unwrap();

        Self {
            message_receiver,
            tcp_handler,
            torrent,
            client_id: generate_random_identity(),
            seeder_peers: HashMap::new(),
            peers_bitfield: HashMap::new(),
            leecher_peers: HashMap::new(),
            bitfield,
            requested_pieces: BitVec::from_elem(bitfield_length, false),
            block_reader_writer,
            blocks_by_piece: HashMap::new(),
            mock_peers,
        }
    }

    fn client_id(&self) -> [u8; 20] {
        self.client_id
    }

    pub fn run(&mut self) {
        log::info!("Starting main loop");

        if self.is_file_on_disk() {
            log::info!("File already on disk");
        }

        self.connect_to_tracker();

        loop {
            self.handle_current_downloads();

            if let Ok((peer, message)) = self.message_receiver.recv() {
                log::debug!("Received message {:?} from {:?}", message, peer);
                self.handle_messsage(peer, message);
            }
        }
    }

    fn is_file_on_disk(&self) -> bool {
        self.bitfield.iter().filter(|piece| *piece).count()
            == self.torrent.number_of_pieces() as usize
    }

    fn handle_messsage(&mut self, peer: Peer, message: Message) {
        let peer_download_state = self.seeder_peers.get(&peer);
        let peer_upload_state = self.leecher_peers.get(&peer);

        match (peer_download_state, peer_upload_state) {
            (None, None) => self.handle_handshake(peer, message),
            (Some(MyLeecherState::WaitingHandshake), _) => self.handle_handshake(peer, message),
            (Some(MyLeecherState::WaitingBitfield | MyLeecherState::BitfieldSent), _) => {
                self.handle_bitfield(peer, message)
            }
            (Some(MyLeecherState::InterestedAndChoked), _) => self.handle_unchoke(peer, message),
            (Some(MyLeecherState::InterestedAndUnchoked), _) => self.handle_piece(peer, message),
            (_, Some(MySeederState::NotInterestingAndChoking)) => {
                self.handle_interested(peer, message)
            }
            (_, Some(MySeederState::InterestingAndUnchoking)) => self.handle_request(peer, message),
            _ => unimplemented!(),
        }
    }

    fn handle_current_downloads(&mut self) {
        let selector = DistributedSelector::pieces_selection(
            self.bitfield.clone(),
            self.peers_bitfield.clone(),
        );

        self.seeder_peers
            .clone()
            .iter()
            .for_each(|(peer, state)| match state {
                MyLeecherState::Unconnected => self.begin_peer_connections(),
                MyLeecherState::InterestedAndUnchoked => {
                    self.request_pieces(peer.clone(), &selector)
                }
                _ => (),
            });
    }

    fn handle_handshake(&mut self, peer: Peer, message: Message) {
        log::debug!("Handling handshake");

        match message {
            Message::Handshake(_message) => {
                if self.is_connection_started(peer) {
                    self.seeder_peers.insert(peer, MyLeecherState::WaitingBitfield);
                } else {
                    self.answer_handshake(peer);
                    self.leecher_peers.insert(peer, MySeederState::NotInterestingAndChoking);
                    self.seeder_peers.insert(peer, MyLeecherState::BitfieldSent);
                }
            },
            _ => log::warn!("Unexpected message from {:?}, cannot initiate a connection without a Handshake message", peer)
        }
    }

    //  If we as a leecher, start a connection sending a handshake
    //  We will also update the leecher_peers list in order to know
    //  If the arriving handshake is from a passive or active connection
    // The objective of this mess, is to not send a bitfield message twice
    // which will be the case if
    fn is_connection_started(&self, peer: Peer) -> bool {
        let is_connected;
        match self.leecher_peers.get(&peer) {
            Some(_) => is_connected = true,
            None => is_connected = false,
        }

        is_connected
    }

    fn handle_bitfield(&mut self, peer: Peer, message: Message) {
        log::debug!("Handling bitfield");
        match message {
            Message::Bitfield(message) => {

                if let Some(MyLeecherState::WaitingBitfield) = self.seeder_peers.get(&peer){
                    self.send_bitfield_message(peer);
                }

                self.seeder_peers.insert(peer, MyLeecherState::NotInterestedAndChoked);
                self.leecher_peers.insert(peer, MySeederState::NotInterestingAndChoking);

                self.peers_bitfield.insert(peer,message.bitfield().clone());
                if self.peers_bitfield.len() == self.seeder_peers.len() {
                    log::info!("All bitfields received");
                    self.handle_all_bitfields();
                } else {
                    let missing_bitfields = self.seeder_peers.len() - self.peers_bitfield.len();
                    log::info!("Waiting for missing {} bitfields", missing_bitfields);
                }
            },
            _ => log::warn!("Unexpected message from peer {:?}, waiting for Handshake response or Bitfield message", peer)
        }
    }

    fn handle_unchoke(&mut self, peer: Peer, message: Message) {
        match message {
            Message::Unchoke(_message) => {
                self.seeder_peers
                    .insert(peer, MyLeecherState::InterestedAndUnchoked);
            }
            _ => log::warn!(
                "Unexpected message from peer {:?}, waiting for Unchoke message",
                peer
            ),
        }
    }

    fn handle_interested(&mut self, peer: Peer, message: Message) {
        log::debug!("Handling Interested message");
        match message {
            Message::Interested(_message) => {
                self.leecher_peers
                    .insert(peer, MySeederState::InterestingAndUnchoking);
                self.send_unchoke_message(peer.clone());
            }
            _ => log::warn!(
                "Unexpected message from peer {:?}, it has not been unchoked",
                peer
            ),
        }
    }

    fn handle_all_bitfields(&mut self) {
        self.update_all_my_leecher_states(MyLeecherState::NotInterestedAndChoked);
        self.send_interested_messages();
    }

    fn answer_handshake(&mut self, peer: Peer) {
        self.send_handshake_message(peer);
        if self.bitfield.any() {
            self.send_bitfield_message(peer);
        }
    }

    fn request_pieces(&mut self, peer: Peer, selector: &HashMap<u32, Option<Peer>>) {
        let mut pieces_to_request = 1;

        selector
            .iter()
            .filter(|(_, maybe_peer)| *maybe_peer == &Some(peer))
            .for_each(|(piece, peer)| {
                if pieces_to_request > 0 {
                    self.request_piece(*piece, peer.unwrap());
                    pieces_to_request -= 1;
                }
            });
    }

    fn request_piece(&mut self, piece_index: u32, peer: Peer) {
        if let Some(true) = self.requested_pieces.get(piece_index as usize) {
            return;
        }

        let blocks_per_piece = torrent::expected_blocks_in_piece(piece_index, &self.torrent) as u32;

        for block in 0..blocks_per_piece {
            let block_length = torrent::expected_block_length(piece_index, block, &self.torrent);

            let request = Request::new(
                piece_index as u32,
                block as u32 * BlockReaderWriter::BIT_TORRENT_BLOCK_SIZE as u32,
                block_length,
            );

            log::info!(
                "Requesting block 0x{:x} from piece {:?} to {:?}",
                block as u32 * BlockReaderWriter::BIT_TORRENT_BLOCK_SIZE as u32,
                piece_index,
                peer
            );

            self.send_message(peer, Message::Request(request));
        }

        self.requested_pieces.set(piece_index as usize, true);
    }

    //TODO: write update_peer_bitfield function at a Have message in any state
    fn handle_piece(&mut self, peer: Peer, message: Message) {
        match message {
            Message::Piece(piece) => {
                self.save_piece(&piece);
                self.print_download_status(&piece);
                let piece_index = piece.piece_index();

                if let Some(true) = self.bitfield.get(piece_index as usize) {
                    let have = Message::Have(Have::new(piece_index));
                    self.send_message(peer, have);
                }

                if !self.is_peer_still_interesting(peer) {
                    self.finish_download_with_peer(peer)
                }
            }
            // Message::Have(message) => self.update_peer_bitfield(peer),
            _ => log::warn!("Unexpected message, waiting for piece."),
        }
    }

    fn print_download_status(&self, piece: &Piece) {
        log::debug!(
            "Received piece {}@{:x}",
            piece.piece_index(),
            piece.begin_offset_of_piece()
        );

        let expected_pieces = self.torrent.number_of_pieces() as usize;
        let received_pieces = self.bitfield.iter().filter(|p| *p).count();
        let percent = (received_pieces as f32 / expected_pieces as f32) * 100.0;

        log::info!(
            "Downloaded {}/{} [{:.2}%]",
            received_pieces,
            expected_pieces,
            percent
        )
    }

    fn save_piece(&mut self, piece: &Piece) {
        self.block_reader_writer
            .write(
                piece.piece_index(),
                piece.begin_offset_of_piece(),
                piece.data(),
            )
            .unwrap();

        if let Some(blocks) = self.blocks_by_piece.get_mut(&piece.piece_index()) {
            *blocks = *blocks + 1;

            let expected_blocks_in_piece =
                torrent::expected_blocks_in_piece(piece.piece_index(), &self.torrent);

            if *blocks == expected_blocks_in_piece {
                self.bitfield.set(piece.piece_index() as usize, true);
            }
        } else {
            self.blocks_by_piece.insert(piece.piece_index(), 1);
        }
    }

    fn handle_request(&self, peer: Peer, message: Message) {
        log::debug!("Handling request");
        match message {
            Message::Request(request) => {
                if self.is_piece_on_disk(request.piece_index()) {
                    self.send_piece(peer, request)
                }
            }
            _ => log::warn!(
                "Unexpected message {:?}, waiting for request, have, or not interested messages",
                message
            ),
        }
    }

    fn send_interested_messages(&mut self) {
        self.peers_bitfield
            .clone()
            .into_iter()
            .for_each(|(peer, peer_bitfield)| {
                let interesting_pieces = self.interesting_pieces(peer_bitfield);

                if interesting_pieces.any() {
                    self.send_interested_message(peer.clone());
                    self.seeder_peers
                        .insert(peer, MyLeecherState::InterestedAndChoked);
                }
            })
    }

    fn begin_peer_connections(&mut self) {
        let peers = self.peers_to_connect();

        for peer in peers {
            if peer.socket_address() == "127.0.0.1:6882" {
                log::warn!("Skipping connection to ourselves.");
                continue;
            }
            //send handshake
            if let Ok(_) = self.connect(peer) {
                self.send_handshake_message(peer);

                //update peer state
                self.seeder_peers
                    .insert(peer, MyLeecherState::WaitingHandshake);
                self.leecher_peers
                    .insert(peer, MySeederState::WaitingHandshake);

                log::debug!("Handshake sent to peer {:?}", peer);
            }
        }
    }

    fn interesting_pieces(&self, mut peer_bitfield: BitVec) -> BitVec {
        let mut local_bitfield = self.bitfield.clone();
        local_bitfield.negate();
        let _ = peer_bitfield.and(&local_bitfield);

        peer_bitfield
    }

    fn mock_peers(&mut self) {
        use std::net::SocketAddr;

        for id in 1..=3 {
            let address = format!("127.0.0.1:200{}", id);
            let peer = Peer::from_socket_address(address.parse::<SocketAddr>().unwrap());
            let initial_state = MyLeecherState::Unconnected;

            self.seeder_peers.insert(peer, initial_state);
        }
    }

    fn connect_to_tracker(&mut self) {
        if self.mock_peers {
            self.mock_peers();
        } else {
            loop {
                match self.send_tracker_request() {
                    Ok(response) => {
                        if !(self.is_file_on_disk()) {
                            self.fill_peer_list(response.peers()).unwrap();
                        }

                        break;
                    }
                    Err(e) => {
                        log::error!("{:?}", e);
                        thread::sleep(Duration::from_secs(1))
                    }
                }
            }
        }
    }

    fn peers_to_connect(&self) -> Vec<Peer> {
        let mut peers = Vec::new();
        for (peer, download_state) in self.seeder_peers.iter() {
            match download_state {
                MyLeecherState::Unconnected => peers.push(*peer),
                _ => (),
            }
        }

        peers
    }

    // Operation
    // peer_bitfield AND !my_bitfield
    fn is_peer_still_interesting(&self, peer: Peer) -> bool {
        let mut my_bitfield = self.bitfield.clone();
        my_bitfield.negate();

        let mut peer_bitfield = self.peers_bitfield.get(&peer).unwrap().clone();
        peer_bitfield.and(&my_bitfield);

        peer_bitfield.any()
    }

    fn finish_download_with_peer(&mut self, peer: Peer) {
        log::info!("{:?} sent all the pieces we needed from it.", peer);
        self.send_not_interested_message(peer);
        self.seeder_peers
            .insert(peer, MyLeecherState::NotInterestedAndUnchoked);
    }

    fn send_handshake_message(&mut self, peer: Peer) {
        let handshake = Handshake::new(self.torrent.info_hash(), self.client_id);
        self.send_message(peer, Message::Handshake(handshake));
    }

    fn send_bitfield_message(&mut self, peer: Peer) {
        let my_bitfield = Bitfield::new(self.bitfield.clone());
        self.send_message(peer, Message::Bitfield(my_bitfield));
    }

    fn send_interested_message(&mut self, peer: Peer) {
        let interested = Interested::new();
        self.send_message(peer, Message::Interested(interested));
    }

    fn send_not_interested_message(&mut self, peer: Peer) {
        let message = NotInterested::new();
        self.send_message(peer, Message::NotInterested(message));
    }

    fn send_unchoke_message(&mut self, peer: Peer) {
        let message = Unchoke::new();
        self.send_message(peer, Message::Unchoke(message));
    }

    fn send_piece(&self, peer: Peer, request: Request) {
        let piece_index = request.piece_index();
        let piece_offset = request.begin_offset();

        let data = self
            .block_reader_writer
            .read(piece_index, piece_offset)
            .unwrap();

        let piece = Piece::new(piece_index, piece_offset, data);
        self.send_message(peer, Message::Piece(piece));
    }

    fn is_piece_on_disk(&self, piece_index: u32) -> bool {
        let may_have_piece = self.bitfield.get(piece_index as usize);
        match may_have_piece {
            Some(true) => return true,
            Some(false) => return false,
            None => {
                log::warn!("Dropping request, piece index {:?} on request message out of bounds, maximal bitfield size is {:?}", piece_index, self.bitfield.len());
                return false;
            }
        }
    }

    /// Sends a message to an already connected peer.
    fn send_message(&self, peer: Peer, message: Message) {
        self.tcp_handler.send((peer, message));
    }

    /// Tries to initiate a connection with a peer.
    fn connect(&mut self, peer: Peer) -> Result<(), Error> {
        self.tcp_handler.connect(peer)
    }

    fn send_tracker_request(&mut self) -> Result<TrackerResponse, Error> {
        // I think for now it's ok to assume that we either have the whole file or
        // nothing at all when the client starts. We should read from disk to check what
        // is the case and fill `left_to_download` with 0 or torrent_size. Based on that,
        // we can decide whether or not we'll iterate the list of peers sent by the tracker
        // to find a peer that has the file we want (if we want a file). If we are seeding,
        // we don't need to do that, and we just need to wait for a handshake instead. The
        // TcpHandler module also listen for connections and the Handshake will be received
        // in the function `run` naturally.
        let torrent = &self.torrent;
        let left_to_download = if self.is_file_on_disk() {
            0
        } else {
            torrent.total_length_in_bytes()
        };

        let tracker_request =
            TrackerRequest::from_torrent(torrent, self.client_id(), left_to_download);
        let tracker_address = TrackerAddress::from_torrent(&self.torrent)?;
        log::debug!("Sending tracker request {:?}", tracker_request);

        let response = TrackerRequest::send_request(tracker_request, tracker_address);
        log::debug!("Tracker response: {:?}", response);

        response
    }

    fn fill_peer_list(&mut self, tracker_peer_list: Option<&Vec<Peer>>) -> Result<(), Error> {
        match tracker_peer_list {
            Some(peers) => {
                for peer in peers {
                    if peer.socket_address() == "127.0.0.1:6882" {
                        log::warn!("Not adding ourselves to the peer list.");
                        continue;
                    }
                    self.seeder_peers.insert(*peer, MyLeecherState::Unconnected);
                }
                log::debug!("Seeding peers: {:?}", self.seeder_peers);
            }

            None => {
                log::info!("Tracker did not return any peers");
                return Err(Error::NoPeersAvailable);
            }
        }
        Ok(())
    }

    fn update_all_my_leecher_states(&mut self, updated_state: MyLeecherState) {
        self.seeder_peers.values_mut().for_each(|state| {
            *state = updated_state.clone();
        });
    }
}
