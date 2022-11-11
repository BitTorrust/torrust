#[cfg(test)]
pub mod unitest {
    use std::net::{Ipv4Addr, SocketAddrV4};

    use crate::http::Peer;
    use crate::pwp::{Handshake, IntoBytes};
    use crate::tcp::TCPSession;
    use std::process::Command;

    const INFO_ID: [u8; 20] = [
        0x06, 0x71, 0x33, 0xac, 0xe5, 0xdd, 0x0c, 0x50, 0x27, 0xb9, 0x9d, 0xe5, 0xd4, 0xba, 0x51,
        0x28, 0x28, 0x20, 0x8d, 0x5b,
    ];

    const PEER_ID: [u8; 20] = [
        0x2d, 0x42, 0x45, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x6e, 0x9a, 0xb4, 0x40, 0x2c, 0x62, 0x2e,
        0x2e, 0x7a, 0x71, 0x5d, 0x9d,
    ];

    #[ignore]
    #[test]
    pub fn connect_to_peer() {
        // Init local network
        Command::new("scripts/seed-icerberg.sh")
            .arg("&")
            .output()
            .expect("failed to execute process");

        // Actual test
        let peer = Peer::from_socket_address(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 2), 6970));
        let tcp_session = TCPSession::connect(peer).unwrap();
        let handshake = Handshake::new(INFO_ID, PEER_ID);
        assert_eq!(tcp_session.send(handshake).unwrap(), 68);

        let mut response: Vec<u8> = Vec::new();
        assert_eq!(tcp_session.receive(&mut response).unwrap(), 68);
    }
}
