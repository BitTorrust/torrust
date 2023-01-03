use {
    crate::{
        error::Error,
        file_management::local_bitfield,
        http::Peer,
        http::{TrackerAddress, TrackerRequest},
        pwp::{Handshake, Interested, Message, Request},
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
    pieces_selection::{DistributedSelector, PiecesSelection},
    BlockReaderWriter, NotInterested,
};

#[derive(Debug)]
pub struct StateMachine {
    message_receiver: Receiver<(Peer, Message)>,
    tcp_handler: TcpHandler,
    torrent: Torrent,
    client_id: [u8; 20],
    download_peers: HashMap<Peer, DownloadBitTorrentState>,
    peers_bitfield: HashMap<Peer, BitVec>,
    upload_peers: HashMap<Peer, UploadBitTorrentState>,
    bitfield: BitVec,
    requested_pieces: BitVec,
    block_reader_writer: BlockReaderWriter,
    blocks_by_piece: HashMap<u32, usize>,
    mock_peers: bool,
}

#[derive(Debug, Clone)]

enum UploadBitTorrentState {
    //Upload states
    WaitHandshake,
    NotInterestingAndChoking,
    InterestingAndChoking,
    InterestingAndUnchoking,
    Done,
}

#[derive(Debug, Clone)]
enum DownloadBitTorrentState {
    //Download states
    Unconnected,
    HandshakeSent,
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

        let filename = working_directory.join(torrent.name());
        let piece_length = torrent.piece_length_in_bytes();
        let file_size = torrent.total_length_in_bytes();
        let block_reader_writer =
            BlockReaderWriter::new(&filename, piece_length, file_size as usize).unwrap();

        Self {
            message_receiver,
            tcp_handler,
            torrent,
            client_id: generate_random_identity(),
            download_peers: HashMap::new(),
            peers_bitfield: HashMap::new(),
            upload_peers: HashMap::new(),
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
        if self.is_download_finished() {
            log::info!("Download already done.");
            return;
        }

        self.fill_peer_list();
        self.send_handshake();

        loop {
            if let Ok((peer, message)) = self.message_receiver.recv() {
                log::debug!("Received message {:?} from {:?}", message, peer);
                self.handle_messsage(peer, message)
            }

            self.handle_current_downloads();

            if self.is_download_finished() {
                self.update_download_peers();
                log::info!("Download finished.");
                break;
            }
        }
    }

    fn is_download_finished(&self) -> bool {
        self.bitfield.iter().filter(|piece| *piece).count()
            == self.torrent.number_of_pieces() as usize
    }

    fn update_download_peers(&mut self) {
        self.download_peers.values_mut().for_each(|state| {
            *state = DownloadBitTorrentState::NotInterestedAndUnchoked;
        });
        self.handle_current_downloads();
    }

    fn handle_messsage(&mut self, peer: Peer, message: Message) {
        if !self.download_peers.contains_key(&peer) {
            return;
        }

        let peer_state = self.download_peers.get(&peer);

        match peer_state {
            Some(peer_state) => match peer_state {
                DownloadBitTorrentState::HandshakeSent => self.handshake_sent(peer, message),
                DownloadBitTorrentState::InterestedAndChoked => self.handle_unchoke(peer, message),
                DownloadBitTorrentState::InterestedAndUnchoked => self.save_piece(message),
                _ => unimplemented!(),
            },
            None => {
                log::info!("Tracker did not return any peers");
            }
        }
    }

    fn handle_current_downloads(&mut self) {
        let selector = DistributedSelector::pieces_selection(
            self.bitfield.clone(),
            self.peers_bitfield.clone(),
        );

        self.download_peers
            .clone()
            .iter()
            .for_each(|(peer, state)| match state {
                DownloadBitTorrentState::InterestedAndUnchoked => {
                    self.request_pieces(peer.clone(), &selector)
                }
                DownloadBitTorrentState::NotInterestedAndUnchoked => {
                    self.send_not_interested(peer.clone())
                }
                _ => (),
            })
    }

    fn handshake_sent(&mut self, peer: Peer, message: Message) {
        log::debug!("Handshake sent state!");

        match message {
            Message::Handshake(message) => {
                log::debug!("Handshake response received from peer {:?}", peer);
            },
            Message::Bitfield(message) => {
                log::debug!("Bitfield message received from peer {:?}", peer);
                self.peers_bitfield.insert(peer,message.bitfield().clone());
                if self.peers_bitfield.len() == self.download_peers.len() {
                    log::info!("All bitfields received");
                    self.download_peers.values_mut().for_each(|v| *v = DownloadBitTorrentState::NotInterestedAndChoked);
                    self.send_interested_messages();
                } else {
                    let missing_bitfields = self.download_peers.len() - self.peers_bitfield.len();
                    log::info!("Waiting for missing {} bitfields", missing_bitfields);
                }
            },
            _ => log::warn!("Unexpected message from peer {:?}, waiting for Handshake response or Bitfield message", peer)
        }
    }

    fn handle_unchoke(&mut self, peer: Peer, message: Message) {
        match message {
            Message::Unchoke(_message) => {
                log::info!("Unchoke message received from peer {:?}", peer);
                self.download_peers
                    .insert(peer, DownloadBitTorrentState::InterestedAndUnchoked);
            }
            _ => log::warn!(
                "Unexpected message from peer {:?}, waiting for Unchoke message",
                peer
            ),
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

        let blocks_per_piece = torrent::div_ceil(
            self.torrent.piece_length_in_bytes(),
            BlockReaderWriter::BIT_TORRENT_BLOCK_SIZE as u32,
        );

        for block in 0..blocks_per_piece {
            let is_last_piece = piece_index as u32 == (self.torrent.number_of_pieces() - 1);
            let is_last_block = block == blocks_per_piece - 1;
            let length = if is_last_piece && is_last_block {
                self.torrent.total_length_in_bytes()
                    % BlockReaderWriter::BIT_TORRENT_BLOCK_SIZE as u32
            } else {
                BlockReaderWriter::BIT_TORRENT_BLOCK_SIZE as u32
            };

            let request = Request::new(
                piece_index as u32,
                block * BlockReaderWriter::BIT_TORRENT_BLOCK_SIZE as u32,
                length,
            );

            self.send_message(peer, Message::Request(request));
        }

        self.requested_pieces.set(piece_index as usize, true);
    }

    fn send_not_interested(&mut self, peer: Peer) {
        let message = NotInterested::new();
        self.send_message(peer, Message::NotInterested(message));
    }
    fn save_piece(&mut self, message: Message) {
        match message {
            Message::Piece(piece) => {
                self.block_reader_writer
                    .write(
                        piece.piece_index(),
                        piece.begin_offset_of_piece(),
                        piece.data(),
                    )
                    .unwrap();
                if let Some(blocks) = self.blocks_by_piece.get_mut(&piece.piece_index()) {
                    *blocks = *blocks + 1;
                    if *blocks
                        == self.torrent.piece_length_in_bytes() as usize
                            / BlockReaderWriter::BIT_TORRENT_BLOCK_SIZE as usize
                            - 1
                    {
                        self.bitfield.set(piece.piece_index() as usize, true);
                    }
                } else {
                    self.blocks_by_piece.insert(piece.piece_index(), 0);
                }
            }
            _ => log::warn!("Unexpected message, waiting for piece."),
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
                    self.download_peers
                        .insert(peer, DownloadBitTorrentState::InterestedAndChoked);
                }
            })
    }

    fn send_interested_message(&mut self, peer: Peer) {
        let interested = Interested::new();
        self.send_message(peer.clone(), Message::Interested(interested));
        log::info!("Interested message sent to {:?}", peer);
    }
    fn interesting_pieces(&self, mut peer_bitfield: BitVec) -> BitVec {
        let mut local_bitfield = self.bitfield.clone();
        local_bitfield.negate();
        let _ = peer_bitfield.and(&local_bitfield);

        peer_bitfield
    }

    fn send_handshake(&mut self) {
        let peers = self.peers_to_handshake();

        for peer in peers {
            if peer.socket_address() == "127.0.0.1:6882" {
                log::warn!("Skipping connection to ourselves.");
                continue;
            }
            //send handshake
            if let Ok(_) = self.connect(peer) {
                let handshake = Handshake::new(self.torrent.info_hash(), self.client_id);
                self.send_message(peer.clone(), Message::Handshake(handshake));

                //update peer state
                self.download_peers
                    .insert(peer, DownloadBitTorrentState::HandshakeSent);

                log::debug!("Handshake sent to peer {:?}", peer);
            }
        }
    }

    fn mock_peers(&mut self) {
        use std::net::SocketAddr;

        for id in 1..=3 {
            let address = format!("127.0.0.1:200{}", id);
            let peer = Peer::from_socket_address(address.parse::<SocketAddr>().unwrap());
            let initial_state = DownloadBitTorrentState::Unconnected;

            self.download_peers.insert(peer, initial_state);
        }
    }

    fn fill_peer_list(&mut self) {
        if self.mock_peers {
            self.mock_peers();
        } else {
            while let Err(_) = self.send_tracker_request() {
                let _ = self.send_tracker_request();
                thread::sleep(Duration::from_millis(500));
            }
        }
    }

    fn peers_to_handshake(&self) -> Vec<Peer> {
        let mut peers = Vec::new();
        for (peer, download_state) in self.download_peers.iter() {
            match download_state {
                DownloadBitTorrentState::Unconnected => peers.push(*peer),
                _ => (),
            }
        }

        peers
    }

    /// Sends a message to an already connected peer.
    fn send_message(&self, peer: Peer, message: Message) {
        log::debug!("Send message: {:?}, to peer: {:?}", peer, message);
        self.tcp_handler.send((peer, message));
    }

    /// Tries to initiate a connection with a peer.
    fn connect(&mut self, peer: Peer) -> Result<(), Error> {
        self.tcp_handler.connect(peer)
    }

    fn send_tracker_request(&mut self) -> Result<(), Error> {
        // I think for now it's ok to assume that we either have the whole file or
        // nothing at all when the client starts. We should read from disk to check what
        // is the case and fill `left_to_download` with 0 or torrent_size. Based on that,
        // we can decide whether or not we'll iterate the list of peers sent by the tracker
        // to find a peer that has the file we want (if we want a file). If we are seeding,
        // we don't need to do that, and we just need to wait for a handshake instead. The
        // TcpHandler module also listen for connections and the Handshake will be received
        // in the function `run` naturally.
        let torrent = &self.torrent;
        // TODO: read from disk/check integrity to see if we have the whole file or nothing.
        let left_to_download = torrent.total_length_in_bytes();

        let tracker_request =
            TrackerRequest::from_torrent(torrent, self.client_id(), left_to_download);
        let tracker_address = TrackerAddress::from_torrent(&self.torrent)?;
        log::debug!("Sending tracker request {:?}", tracker_request);

        let response = TrackerRequest::send_request(tracker_request, tracker_address)?;
        log::debug!("Tracker response: {:?}", response);

        // TODO: decide what to do with the tracker response, based on whether or not we'll
        // need to leech.

        // en assumant leecher maintenant
        match response.peers() {
            Some(peers) => {
                for peer in peers {
                    if peer.socket_address() == "127.0.0.1:6882" {
                        log::warn!("Not adding ourselves to the peer list.");
                        continue;
                    }

                    self.download_peers
                        .insert(*peer, DownloadBitTorrentState::Unconnected);
                }
            }

            None => {
                log::info!("Tracker did not return any peers");
                return Err(Error::NoPeersAvailable);
            }
        }

        Ok(())
    }
}
