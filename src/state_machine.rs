use {
    crate::{error::Error, http::Peer, pwp::Message, tcp::TCPSessionMock},
    crossbeam_channel::{Receiver, Sender},
    log::warn,
    std::{
        collections::HashMap,
        net::{SocketAddr, TcpListener, TcpStream},
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    },
};

#[derive(Debug)]
pub struct StateMachine {
    message_receiver: Receiver<(Peer, Message)>,
    tcp_handler: TcpHandler,
    // structure to store the state of each peer? HashMap<Peer, BitTorrentState>
}

impl StateMachine {
    pub fn new() -> Self {
        let (message_sender, message_receiver) = crossbeam_channel::unbounded();
        let tcp_handler = TcpHandler::new(message_sender);

        Self {
            message_receiver,
            tcp_handler,
        }
    }

    pub fn run(&self) {
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
}

#[derive(Debug)]
pub struct TcpHandler {
    peers: Arc<Mutex<HashMap<Peer, TCPSessionMock>>>,
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
        let stream =
            TcpStream::connect(peer.socket_address()).map_err(|_| Error::FailedToConnectToPeer)?;

        // let tcp_session = TCPSession::from_stream
        self.peers.lock().unwrap().insert(peer, unimplemented!());

        Ok(())
    }

    pub fn send(&self, message: (Peer, Message)) {
        self.tcp_sender.send(message).unwrap();
    }

    pub fn run(
        peers: Arc<Mutex<HashMap<Peer, TCPSessionMock>>>,
        message_sender: Sender<(Peer, Message)>,
        tcp_receiver: Receiver<(Peer, Message)>,
    ) {
        log::debug!("TcpHandler loop started");

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
        peers: Arc<Mutex<HashMap<Peer, TCPSessionMock>>>,
        message_receiver: Receiver<(Peer, Message)>,
    ) {
        while let Ok((peer, message)) = message_receiver.recv() {
            let message = message.into_inner();
            peers
                .lock()
                .unwrap()
                .get(&peer)
                .unwrap()
                .send_bytes(message.into_bytes())
                .unwrap();
        }
    }

    /// This function spawns a thread that continously listen for external
    /// connections. A connection of this kind happens when a peer wants
    /// a file that we are seeding. The functionn accepts connections and
    /// insert the respective TcpSession into the Peers hashmap.
    fn connection_listener(peers: Arc<Mutex<HashMap<Peer, TCPSessionMock>>>) {
        log::debug!("Listening for connections");

        // TODO: check the port
        let tcp_listener = TcpListener::bind("0.0.0.0:6969").unwrap();

        for stream in tcp_listener.incoming() {
            let stream = stream.unwrap();
            match stream.peer_addr().unwrap() {
                SocketAddr::V4(address_v4) => {
                    log::info!("Peer {} initiated a connection.", address_v4);
                    let peer = Peer::from_socket_address(address_v4);

                    // let tcp_session = TCPSession::from_stream
                    peers.lock().unwrap().insert(peer, unimplemented!());
                }
                _ => {
                    warn!("Skipping client trying to use IPv6.");
                    break;
                }
            };
        }
    }
}
