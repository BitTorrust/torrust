use {
    crate::{
        error::Error,
        http::Peer,
        http::{TrackerAddress, TrackerRequest},
        pwp::{Handshake, Message},
        torrent::Torrent,
    },
    crossbeam_channel::Receiver,
    std::path::PathBuf,
};

mod tcp_handler;
use std::{collections::HashMap, hash::Hash};

use tcp_handler::TcpHandler;

mod wait;
pub use wait::Wait;

pub(crate) mod identity;
use identity::generate_random_identity;

#[derive(Debug)]
pub struct StateMachine {
    message_receiver: Receiver<(Peer, Message)>,
    tcp_handler: TcpHandler,
    torrent: Torrent,
    client_id: [u8; 20],
    download_peers: HashMap<Peer, DownloadBitTorrentState>,
    upload_peers: HashMap<Peer, UploadBitTorrentState>,
    // structure to store the state of each peer? HashMap<Peer, BitTorrentState>
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
        Self {
            message_receiver,
            tcp_handler,
            torrent,
            client_id: generate_random_identity(),
            download_peers: HashMap::new(),
            upload_peers: HashMap::new(),
        }
    }

    fn client_id(&self) -> [u8; 20] {
        self.client_id
    }

    pub fn run(&mut self) {
        self.fill_peer_list();

        self.send_handshake();

        //1. Handle message function (peer, message)
        //2. Got it.

        while let Ok((peer, message)) = self.message_receiver.recv() {
            log::debug!("Received message {:?} from {:?}", message, peer);
            // TODO: match and handle message

            // if self.download_peers.contains_key(&peer) {
            //     let peer_state = self.download_peers.get(&peer);

            //     match peer_state {
            //         Some(peer_state) => match peer_state {
            //             DownloadBitTorrentState::HandshakeSent => {
            //                 self.handshake_sent(peer, message)
            //             }
            //             _ => unimplemented!(),
            //         },
            //         None => {
            //             log::info!("Tracker did not return any peers");
            //             // return Err(Error::NoPeersAvailable);
            //             // ();
            //         }
            //     }
            // }
        }
    }

    //TODO : NOT send handshake to ourselves
    fn send_handshake(&mut self) {
        let peers = self.peers_to_handshake();

        for peer in peers {
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

    fn handshake_sent(&self, peer: Peer, message: Message) {
        log::debug!(" Feliz Navidad, pude hacer entrar en un handshake sent");
    }

    /// Sends a message to an already connected peer.
    fn send_message(&self, peer: Peer, message: Message) {
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
