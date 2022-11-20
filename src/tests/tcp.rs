#[cfg(test)]
pub mod user_case {
    use std::{
        fs::{self, File},
        io::{self, Read},
        net::{Ipv4Addr, SocketAddrV4},
        path::Path,
        thread::sleep,
        time::Duration,
    };

    use bendy::decoding::Decoder;
    use bit_vec::BitVec;

    use crate::{
        http::Peer,
        pwp::{FromBytes, Interested, MandatoryBitTorrentMessageFields},
    };
    use crate::{pwp::Bitfield, tcp::TCPSession};
    use crate::{pwp::Handshake, Error, Torrent};
    use std::process::{Child, Command};

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
    /// Seeder  [IP:127.0.0.1,port:6999]
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
            .arg(UPLOAD_FILES_FOLDER.to_string())
            .arg(format!("--listen-port={}", SEEDER_TCP_DOWNLOAD_PORT))
            .spawn()
    }

    #[test]
    pub fn leech() {
        // Init local network
        let filename_to_upload = "venon.jpg";
        let torrent_filename_to_upload = format!("{}.torrent", filename_to_upload);
        let mut tracker_process_child =
            run_tracker().expect("failed to execute tracker process child");

        let mut seeder_process_child = run_seeder(&torrent_filename_to_upload)
            .expect("failed to execute seeder process child");
        sleep(Duration::from_secs(5));

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

        // Leecher --[Handshake]-> Seeder
        let filepath = format!("{}/{}", UPLOAD_FILES_FOLDER, torrent_filename_to_upload);
        let filepath = Path::new(&filepath);
        let mut file = File::open(filepath).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let mut bencode_decoder = Decoder::new(&buffer);
        let maybe_torrent = Torrent::from_bencode(&mut bencode_decoder);
        let info_hash = maybe_torrent.unwrap().info_hash();
        let handshake = Handshake::new(info_hash, PEER_ID);
        let expected_handshake_length_in_byte = 68;
        // Check all the handshake has been sent
        assert_eq!(
            tcp_session.send(handshake).unwrap(),
            expected_handshake_length_in_byte
        );

        // Leecher <-[Handshake]-- Seeder
        let mut handshake_response_bytes: Vec<u8> = vec![0; expected_handshake_length_in_byte];
        let received_hanshake_size = match tcp_session.receive(&mut handshake_response_bytes) {
            Ok(size) => size,
            Err(error) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("{:?}", error);
            }
        };
        println!("HANDSHAKE received: {:?}\n\n", handshake_response_bytes);
        let (received_handshake, size_read_from_hanshake_bytes) =
            match Handshake::from_bytes(&handshake_response_bytes) {
                Ok(handshake_size) => handshake_size,
                Err(error) => {
                    tracker_process_child.kill().unwrap();
                    seeder_process_child.kill().unwrap();
                    panic!("{:?}", error);
                }
            };
        // Check the handshake corresponding to our torrent has been received
        assert_eq!(received_handshake.info_hash(), info_hash);
        assert_eq!(
            size_read_from_hanshake_bytes,
            expected_handshake_length_in_byte
        );
        assert_eq!(expected_handshake_length_in_byte, received_hanshake_size);

        // // Leecher <-[Bitfield]-- Seeder
        // let expected_bitfield_length = (torrent.number_of_pieces().unwrap() + 5) as usize;
        // let mut bitfield_response_bytes: Vec<u8> = vec![0; expected_bitfield_length];
        // let mut received_bitfield_size = 0;
        // println!("waiting to receive bitfield from seeder..");
        // while received_bitfield_size == 0 {
        //     received_bitfield_size = match tcp_session.receive(&mut bitfield_response_bytes) {
        //         Ok(size) => size,
        //         Err(error) => {
        //             tracker_process_child.kill().unwrap();
        //             seeder_process_child.kill().unwrap();
        //             panic!("{:?}", error);
        //         }
        //     };
        //     print!(".");
        // }

        // println!("BITFIELD received {:?}", bitfield_response_bytes);
        // println!("received_bitfield_size {}", received_bitfield_size);
        // let (received_bitfield, received_bitfield_size) =
        //     match Bitfield::from_bytes(&bitfield_response_bytes) {
        //         Ok(bitfield_size) => bitfield_size,
        //         Err(error) => {
        //             tracker_process_child.kill().unwrap();
        //             seeder_process_child.kill().unwrap();
        //             panic!("{:?}", error);
        //         }
        //     };
        // // Check we received the seeder bitfield full
        // let expected_received_bitfield = BitVec::from_bytes(&[
        //     0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        //     0xff, 0xff, 0xff, 0xff, 0x80,
        // ]);
        // assert_eq!(*received_bitfield.bitfield(), expected_received_bitfield);
        // let expected_received_bitfield_size = received_bitfield.message_length() + 4;
        // assert_eq!(
        //     received_bitfield_size as u32,
        //     expected_received_bitfield_size
        // );

        // // Leecher --[Bitfield]-> Seeder
        // let leacher_bitfield = BitVec::from_bytes(&[0x0]);
        // let bitfield = Bitfield::new(leacher_bitfield);
        // let expected_bitfield_message_length = bitfield.message_length() + 4;
        // assert_eq!(
        //     tcp_session.send(bitfield).unwrap() as u32,
        //     expected_bitfield_message_length
        // );

        // // Leecher --[Interested]-> Seeder
        // let interested = Interested::new();
        // let expected_interested_length_in_byte = 5;
        // // Check all the interested has been sent
        // assert_eq!(
        //     tcp_session.send(interested).unwrap(),
        //     expected_interested_length_in_byte
        // );

        // End of test
        tracker_process_child.kill().unwrap();
        seeder_process_child.kill().unwrap();

        let seeder_download_file = format!("{}/{}", DOWNLOAD_FILES_FOLDER, filename_to_upload);
        let _ = fs::remove_file(seeder_download_file);
        let seeder_aria_file = format!("{}/{}.aria2", DOWNLOAD_FILES_FOLDER, filename_to_upload);
        let _ = fs::remove_file(seeder_aria_file);
    }
}
