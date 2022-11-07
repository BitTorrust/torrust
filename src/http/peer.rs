use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Debug)]
pub struct Peer {
    socket_address: SocketAddrV4,
}

impl Peer {
    pub fn from_bytes(chunk: &[u8]) -> Self {
        let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
        let port = (chunk[4] as u16 * 256) + chunk[5] as u16;

        Peer {
            socket_address: SocketAddrV4::new(ip, port),
        }
    }
}
