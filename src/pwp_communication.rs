pub enum PeerToWireState {
    Idle,
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
}
pub struct PeerToWireCommunication {
    state: PeerToWireState,
}

impl PeerToWireCommunication {
    pub fn new() -> Self {
        PeerToWireCommunication {
            state: PeerToWireState::Idle,
        }
    }

    pub fn idle() {
        println!("I am in the idle state");
    }

    pub fn tracker_request_sent() {
        println!("I am in the tracker request sent state");
    }

    pub fn unconnected_with_peers() {
        println!("I am in the unconnected with peers state");
    }

    pub fn handshake_sent() {
        println!("I am in the handshake state");
    }
    pub fn wait_handshake() {
        println!("I am in the wait handshake state");
    }

    pub fn connection_with_peer() {
        println!("I am in the connection with peer state");
    }
    pub fn not_interested_and_choked() {
        println!("I am in the Not interested and choked state");
    }

    pub fn interested_and_choked() {
        println!("I am in the interested and choked state");
    }

    pub fn interested_and_unchoked() {
        println!("I am in the interested and unchoked state");
    }

    pub fn not_interested_and_unchoked() {
        println!("I am in the not interested and unchoked state");
    }

    pub fn not_interesting_and_choking() {
        println!("I am in the not interesting and choking state");
    }

    pub fn interesting_and_choking() {
        println!("I am in the interesting and choking state");
    }

    pub fn interesting_and_unchoking() {
        println!("I am in the interesting and unchoking state");
    }

    pub fn not_interesting_and_unchoking() {
        println!("I am in the not interesting and unchoking state");
    }

    pub fn state_transition(&self) {
        match &self.state {
            Idle => Self::idle(),
            TrackerRequestSent => Self::tracker_request_sent(),
            UnconnectedWithPeers => Self::unconnected_with_peers(),
            HandshakeSent => Self::handshake_sent(),

            NotInterestedAndChoked => Self::not_interested_and_choked(),
            InterestedAndChoked => Self::interested_and_choked(),
            InterestedAndUnchoked => Self::interested_and_unchoked(),
            NotInterestedAndUnchoked => Self::not_interested_and_unchoked(),
        }
    }
}
