use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Debug, Clone, Copy)]
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

    pub fn from_socket_address(socket_address: SocketAddrV4) -> Self {
        Peer { socket_address }
    }

    pub fn socket_address(self) -> String {
        self.socket_address.to_string()
    }
}