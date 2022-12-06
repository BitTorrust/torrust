use {
    crate::{
        error::Error, http::Peer, http::TrackerRequest, pwp::Message,
        pwp_communication::TrackerAddress, tcp::TcpSession, torrent::Torrent,
    },
    crossbeam_channel::{Receiver, Sender},
    std::{
        collections::HashMap,
        net::{TcpListener, TcpStream},
        path::PathBuf,
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    },
};

const PEER_ID: [u8; 20] = [
    0xDE, 0xAD, 0xBE, 0xEF, 0xBA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
    0xAA, 0xAA, 0xAA, 0xAD,
];

#[derive(Debug)]
pub struct StateMachine {
    message_receiver: Receiver<(Peer, Message)>,
    tcp_handler: TcpHandler,
    torrent: Torrent,
    // structure to store the state of each peer? HashMap<Peer, BitTorrentState>
}

impl StateMachine {
    pub fn new(torrent: Torrent, working_directory: &PathBuf) -> Self {
        let (message_sender, message_receiver) = crossbeam_channel::unbounded();
        let tcp_handler = TcpHandler::new(message_sender);

        Self {
            message_receiver,
            tcp_handler,
            torrent,
        }
    }

    pub fn run(&self) {
        self.send_tracker_request().unwrap();

        while let Ok((peer, message)) = self.message_receiver.recv() {
            log::debug!("Received message {:?} from {:?}", message, peer);
            // match and handle message
            // MessageHandler.handle_message(peer, message)
        }
    }

    fn send_message(&self, peer: Peer, message: Message) {
        self.tcp_handler.send((peer, message));
    }

    fn connect(&mut self, peer: Peer) {
        self.tcp_handler.connect(peer).unwrap();
    }

    fn send_tracker_request(&self) -> Result<(), Error> {
        // TODO: I think for now it's ok to assume that we either have the whole file or
        // nothing at all when the client starts. We should read from disk to check what
        // is the case and fill `left_to_download` with 0 or torrent_size.
        let torrent = &self.torrent;
        let left_to_download = torrent.total_length_in_bytes();

        let tracker_request = TrackerRequest::from_torrent(torrent, PEER_ID, left_to_download);
        let tracker_address = TrackerAddress::from_torrent(&self.torrent)?;
        log::debug!("Sending tracker request {:?}", tracker_request);

        let response = TrackerRequest::send_request(tracker_request, tracker_address)?;
        log::debug!("Tracker response: {:?}", response);

        Ok(())
    }
}

#[derive(Debug)]
pub struct TcpHandler {
    peers: Arc<Mutex<HashMap<Peer, TcpSession>>>,
    tcp_sender: Sender<(Peer, Message)>,
}

impl TcpHandler {
    pub fn new(message_sender: Sender<(Peer, Message)>) -> Self {
        let (tcp_sender, tcp_receiver) = crossbeam_channel::unbounded();
        let peers = Arc::new(Mutex::new(HashMap::new()));
        let peers_ref = peers.clone();

        thread::spawn(move || TcpHandler::run(peers_ref, message_sender.clone(), tcp_receiver));

        Self { peers, tcp_sender }
    }

    /// Connect to a Peer and insert it in the hashmap of Peers.
    pub fn connect(&mut self, peer: Peer) -> Result<(), Error> {
        let tcp_session = TcpSession::connect(peer.clone())?;
        self.peers.lock().unwrap().insert(peer, tcp_session);

        Ok(())
    }

    pub fn send(&self, message: (Peer, Message)) {
        self.tcp_sender.send(message).unwrap();
    }

    pub fn run(
        peers: Arc<Mutex<HashMap<Peer, TcpSession>>>,
        message_sender: Sender<(Peer, Message)>,
        tcp_receiver: Receiver<(Peer, Message)>,
    ) {
        log::info!("Thread TcpHandler started.");

        let peers_ref = peers.clone();
        thread::spawn(move || TcpHandler::connection_listener(peers_ref));
        let peers_ref = peers.clone();
        thread::spawn(move || TcpHandler::tcp_sender(peers_ref, tcp_receiver));

        loop {
            // Send messages received through TCP to the state machine
            for (peer, session) in peers.lock().unwrap().iter_mut() {
                while let Some(message) = session.receive().unwrap() {
                    message_sender.send((*peer, message)).unwrap();
                }
            }

            // TODO: create a smart waiter (busy wait -> sleep)
            thread::sleep(Duration::from_millis(10));
        }
    }

    // Send messages received from the StateMachine to the peers
    // recv() message_receiver, get TCPSession and call TCPSEssion.send
    fn tcp_sender(
        peers: Arc<Mutex<HashMap<Peer, TcpSession>>>,
        tcp_receiver: Receiver<(Peer, Message)>,
    ) {
        log::info!("Thread TcpSender started.");

        while let Ok((peer, message)) = tcp_receiver.recv() {
            peers
                .lock()
                .unwrap()
                .get(&peer)
                .unwrap()
                .send(message)
                .unwrap();
        }

        log::info!("Thread TcpSender exited.");
    }

    /// This function spawns a thread that continously listen for external
    /// connections. A connection of this kind happens when a peer wants
    /// a file that we are seeding. The functionn accepts connections and
    /// insert the respective TcpSession into the Peers hashmap.
    fn connection_listener(peers: Arc<Mutex<HashMap<Peer, TcpSession>>>) {
        log::info!("Thread ConnectionListener started.");

        // TODO: check the port. We should use the same port we annouce to the tracker.
        let tcp_listener = TcpListener::bind("0.0.0.0:6882").unwrap();

        for stream in tcp_listener.incoming() {
            let stream = stream.unwrap();
            let address = stream.peer_addr().unwrap();
            let peer = Peer::from_socket_address(address);

            log::info!("Peer {} initiated a connection.", address);
            let tcp_session = TcpSession::from_stream(stream).unwrap();
            peers.lock().unwrap().insert(peer, tcp_session);
        }

        log::info!("Thread ConnectionListener exited.");
    }
}
