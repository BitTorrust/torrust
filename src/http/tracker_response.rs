use std::net::SocketAddrV4;

pub struct Peer {
    socket_address: SocketAddrV4,
}

pub enum TrackerResponse {
    Failure { message: String },
    Success { interval: usize, peers: Vec<Peer> },
}
