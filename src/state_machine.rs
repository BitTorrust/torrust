use {
    crate::{
        error::Error,
        file_management::local_bitfield,
        http::Peer,
        http::{TrackerAddress, TrackerRequest},
        pwp::{Handshake, Message, Interested, Request},
        torrent::Torrent,
    },
    crossbeam_channel::Receiver,
    std::path::PathBuf,
    std::thread,
};

mod tcp_handler;
use std::{collections::HashMap, hash::Hash, time::Duration};

use bit_vec::BitVec;
use tcp_handler::TcpHandler;

mod wait;
pub use wait::Wait;

pub(crate) mod identity;
use identity::generate_random_identity;

use crate::Bitfield;

#[derive(Debug)]
pub struct StateMachine {
    message_receiver: Receiver<(Peer, Message)>,
    tcp_handler: TcpHandler,
    torrent: Torrent,
    client_id: [u8; 20],
    download_peers: HashMap<Peer, DownloadBitTorrentState>,
    peers_bitfield: HashMap<Peer, BitVec>,
    upload_peers: HashMap<Peer, UploadBitTorrentState>,
    bitfield: BitVec, // structure to store the state of each peer? HashMap<Peer, BitTorrentState>
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
}

impl StateMachine {
    pub const CLIENT_PORT: u16 = 6882;

    pub fn new(torrent: Torrent, working_directory: &PathBuf) -> Self {
        let (message_sender, message_receiver) = crossbeam_channel::unbounded();
        let tcp_handler = TcpHandler::new(message_sender);
        let bitfield = local_bitfield(&torrent, working_directory);
        Self {
            message_receiver,
            tcp_handler,
            torrent,
            client_id: generate_random_identity(),
            download_peers: HashMap::new(),
            peers_bitfield: HashMap::new(),
            upload_peers: HashMap::new(),
            bitfield,
        }
    }

    fn client_id(&self) -> [u8; 20] {
        self.client_id
    }

    pub fn run(&mut self) {
        self.fill_peer_list();
        self.send_handshake();

        loop {
            let maybe_message = self.message_receiver.recv_timeout(Duration::from_millis(10));
            
            if let Ok((peer, message)) = maybe_message {
                log::debug!("Received message {:?} from {:?}", message, peer);
                self.handle_messsage(peer, message)  
            }
            
            self.handle_current_downloads();
        }

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
                _ => unimplemented!(),
            },
            None => {
                log::info!("Tracker did not return any peers");
            }
        }
    }

    fn handle_current_downloads(&self) {
        self.download_peers.iter().for_each(|(peer, state)| {
            match state {
                DownloadBitTorrentState::InterestedAndUnchoked => self.request_pieces(peer.clone()),
                _ => ()
            }
        })
    }
    // For multiple peers
    // Resume download (check my bitfield)
    // Compare local bitfield with peer bitfield
    // => envío un interested y guardo cuales son las piezas que me interesan de él
    // => no envio nada y quedo en notInterested.
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
                    self.send_interested_message();
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
            Message::Unchoke(message) => {
                log::info!("Unchoke message received from peer {:?}", peer);
                self.download_peers.insert(peer, DownloadBitTorrentState::InterestedAndUnchoked);
            },
            _ => log::warn!("Unexpected message from peer {:?}, waiting for Unchoke message", peer)
        }
    }

    fn request_pieces(&self, peer: Peer) {
        let mut peer_bitfield = self.peers_bitfield.get(&peer).unwrap();
        let interesting_pieces = self.interesting_pieces(peer_bitfield.clone());

        interesting_pieces.iter().enumerate().filter(|(piece_index, value)| *value).for_each(|(piece_index,_)| {

            let request = Request::new(piece_index as u32, 0, self.torrent.piece_length_in_bytes());
            self.send_message(peer.clone(), Message::Request(request));
            thread::sleep(Duration::from_millis(100));

        })


    }

    fn send_interested_message(&mut self) {
        self.peers_bitfield.clone().into_iter().for_each(|(peer, peer_bitfield)| {
            let interesting_pieces = self.interesting_pieces(peer_bitfield);

            if interesting_pieces.any() {
                let interested = Interested::new();
                self.send_message(peer.clone(), Message::Interested(interested));
                log::info!("Interested message sent to {:?}", peer);
                self.download_peers.insert(peer, DownloadBitTorrentState::InterestedAndChoked);
            }
        })

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

    fn fill_peer_list(&mut self) {
        while let Err(_) = self.send_tracker_request() {
            self.send_tracker_request();
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
