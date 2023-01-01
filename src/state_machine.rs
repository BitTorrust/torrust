use {
    crate::{
        error::Error,
        http::Peer,
        http::{TrackerAddress, TrackerRequest},
        pwp::Message,
        torrent::Torrent,
    },
    crossbeam_channel::Receiver,
    std::path::PathBuf,
};

mod tcp_handler;
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
    // structure to store the state of each peer? HashMap<Peer, BitTorrentState>
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
        }
    }

    fn client_id(&self) -> [u8; 20] {
        self.client_id
    }

    pub fn run(&self) {
        self.send_tracker_request().unwrap();

        while let Ok((peer, message)) = self.message_receiver.recv() {
            log::debug!("Received message {:?} from {:?}", message, peer);
            // TODO: match and handle message
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

    fn send_tracker_request(&self) -> Result<(), Error> {
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

        Ok(())
    }
}
