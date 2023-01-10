use {
    crate::{
        adaptative_wait::AdaptativeWait,
        error::Error,
        http::Peer,
        pwp::Message,
        state_machine::{StateMachine, Wait},
        tcp::TcpSession,
    },
    crossbeam_channel::{Receiver, Sender},
    std::{
        collections::HashMap,
        net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    },
};

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

        let adaptative_wait = AdaptativeWait::new(64, Duration::from_millis(100));
        thread::spawn(move || {
            TcpHandler::run(
                peers_ref,
                message_sender.clone(),
                tcp_receiver,
                adaptative_wait,
            )
        });

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

    /// Starts the TcpHandler, a module that has 3 functions:
    ///    - Accept connections from other peers attempting to leech from us.
    ///    - Send Messages from the StateMachine to the Peers through TCP.
    ///    - Send Messages received from the Peers through TCP to the state machine.
    pub fn run(
        peers: Arc<Mutex<HashMap<Peer, TcpSession>>>,
        message_sender: Sender<(Peer, Message)>,
        tcp_receiver: Receiver<(Peer, Message)>,
        mut wait_mechanism: impl Wait,
    ) {
        log::info!("Thread TcpHandler started.");

        let peers_ref = peers.clone();
        thread::spawn(move || TcpHandler::connection_listener(peers_ref));
        let peers_ref = peers.clone();
        thread::spawn(move || TcpHandler::tcp_sender(peers_ref, tcp_receiver));

        loop {
            let mut messages_to_send = Vec::new();
            // Send messages received through TCP to the state machine
            if let Ok(ref mut peers) = peers.try_lock() {
                for (peer, ref mut session) in peers.iter_mut() {
                    while let Some(message) = session.receive().unwrap() {
                        messages_to_send.push((*peer, message));
                    }
                }
            }

            // Make sure to call send only after dropping the lock, otherwise we may deadlock.
            messages_to_send
                .into_iter()
                .for_each(|(peer, message)| message_sender.send((peer, message)).unwrap());

            wait_mechanism.wait();
        }
    }

    /// Continously send messages received from the StateMachine to the peers
    fn tcp_sender(
        peers: Arc<Mutex<HashMap<Peer, TcpSession>>>,
        tcp_receiver: Receiver<(Peer, Message)>,
    ) {
        log::info!("Thread TcpSender started.");

        while let Ok((peer, message)) = tcp_receiver.recv() {
            let result = peers.lock().unwrap().get(&peer).unwrap().send(message);
            if let Err(_) = result {
                log::warn!("Connection with {:?} is broken.", peer);
            }
        }

        log::info!("Thread TcpSender exited.");
    }

    /// Continously listen for external connections. A connection of this
    /// kind happens when a peer wants a file that we are seeding. The
    /// functionn accepts connections and insert the respective TcpSession
    /// into the Peers hashmap.
    fn connection_listener(peers: Arc<Mutex<HashMap<Peer, TcpSession>>>) {
        log::info!("Thread ConnectionListener started.");

        let client_ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        let socket_address = SocketAddr::new(client_ip, StateMachine::CLIENT_PORT);
        let tcp_listener = TcpListener::bind(socket_address).unwrap();

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
