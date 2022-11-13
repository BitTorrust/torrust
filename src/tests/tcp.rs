#[cfg(test)]
pub mod unitest {
    use std::{
        io,
        net::{Ipv4Addr, SocketAddrV4},
        thread::sleep,
        time::Duration,
    };

    use crate::pwp::Handshake;
    use crate::tcp::TCPSession;
    use crate::{http::Peer, pwp::FromBytes};
    use std::process::{Child, Command};

    const INFO_ID: [u8; 20] = [
        0x06, 0x71, 0x33, 0xac, 0xe5, 0xdd, 0x0c, 0x50, 0x27, 0xb9, 0x9d, 0xe5, 0xd4, 0xba, 0x51,
        0x28, 0x28, 0x20, 0x8d, 0x5b,
    ];

    const PEER_ID: [u8; 20] = [
        0x2d, 0x42, 0x45, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x6e, 0x9a, 0xb4, 0x40, 0x2c, 0x62, 0x2e,
        0x2e, 0x7a, 0x71, 0x5d, 0x9d,
    ];

    const UPLOAD_FILES_FOLDER: &str = "samples/upload";
    const DOWNLOAD_FILES_FOLDER: &str = "samples/download";

    /// Tracker [IP:127.0.0.1,port:6969]
    const OPENTRACKER_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
    const OPENTRACKER_TCP_PORT: u16 = 6969;
    ///     |
    /// Seeder  [IP:127.0.0.1,port:7000]
    const SEEDER_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
    const SEEDER_TCP_DOWNLOAD_PORT: u16 = 6999;
    ///     |
    /// Leecher [port:?]

    fn run_tracker() -> io::Result<Child> {
        Command::new("opentracker")
            .arg("-i")
            .arg(OPENTRACKER_IP_ADDRESS.to_string())
            .arg("-p")
            .arg(OPENTRACKER_TCP_PORT.to_string())
            .spawn()
    }

    fn run_seeder(torrent_to_seed_filename: &str) -> io::Result<Child> {
        Command::new("aria2c")
            .arg(format!(
                "{}/{}",
                UPLOAD_FILES_FOLDER, torrent_to_seed_filename
            ))
            .arg("-V")
            .arg("-d")
            .arg(DOWNLOAD_FILES_FOLDER.to_string())
            .arg(format!("--listen-port={}", SEEDER_TCP_DOWNLOAD_PORT))
            .spawn()
    }

    #[ignore]
    #[test]
    pub fn handshake_to_seeder() {
        // Init local network
        let filename_to_download = "iceberg.jpg.torrent";
        let mut tracker_process_child =
            run_tracker().expect("failed to execute tracker process child");
        sleep(Duration::from_secs(1));

        let mut seeder_process_child =
            run_seeder(filename_to_download).expect("failed to execute seeder process child");
        sleep(Duration::from_secs(1));

        // TCP connection
        let seeder_peer = Peer::from_socket_address(SocketAddrV4::new(
            SEEDER_IP_ADDRESS,
            SEEDER_TCP_DOWNLOAD_PORT,
        ));
        let tcp_session = match TCPSession::connect(seeder_peer) {
            Ok(session) => session,
            Err(_) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("cannot connect to seeder");
            }
        };

        // Leecher --[Handshake]--> Seeder
        let handshake = Handshake::new(INFO_ID, PEER_ID);
        let expected_handshake_length_in_byte = 68;
        assert_eq!(
            tcp_session.send(handshake).unwrap(),
            expected_handshake_length_in_byte
        );

        // Leecher <--[Handshake, Bitfield?]-- Seeder
        let mut response_bytes: Vec<u8> = vec![0; 128];
        let bytes_received_length = match tcp_session.receive(&mut response_bytes) {
            Ok(size) => size,
            Err(error) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("{:?}", error);
            }
        };
        assert_eq!(bytes_received_length, expected_handshake_length_in_byte);

        let received_handshake = match Handshake::from_bytes(&response_bytes) {
            Ok(handshake_size) => handshake_size.0,
            Err(error) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("{:?}", error);
            }
        };
        assert_eq!(received_handshake.info_hash(), INFO_ID);

        tracker_process_child.kill().unwrap();
        seeder_process_child.kill().unwrap();
    }
}
