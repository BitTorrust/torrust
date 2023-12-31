#[cfg(test)]
pub mod user_case {
    use crate::{
        http::Peer,
        pwp::{
            Bitfield, Handshake, Interested, MandatoryBitTorrentMessageFields, Message,
            MessageType, Request,
        },
        tcp::TcpSession,
        tests::pwp::unittest::{path_build_to_pwp_message, read_bytes_from},
        BlockReaderWriter, Torrent,
    };
    use core::panic;
    use std::{
        fs::File,
        io::{self, Read},
        net::{IpAddr, Ipv4Addr, SocketAddr},
        path::Path,
        process::{Child, Command},
        thread::sleep,
        time::Duration,
    };

    use bendy::decoding::Decoder;
    use bit_vec::BitVec;

    const PEER_ID: [u8; 20] = [
        0x2d, 0x42, 0x45, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x6e, 0x9a, 0xb4, 0x40, 0x2c, 0x62, 0x2e,
        0x2e, 0x7a, 0x71, 0x5d, 0x9d,
    ];

    const UPLOAD_FILES_FOLDER: &str = "samples/upload";

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

    fn run_seeder(torrent_to_seed_filename: &str, port_to_seed: u16) -> io::Result<Child> {
        Command::new("aria2c")
            .arg(format!(
                "{}/{}",
                UPLOAD_FILES_FOLDER, torrent_to_seed_filename
            ))
            .arg("-V")
            .arg("-d")
            .arg(UPLOAD_FILES_FOLDER.to_string())
            .arg(format!("--listen-port={}", port_to_seed))
            .spawn()
    }

    #[ignore = "CI is not setup for integration test using network"]
    #[test]
    pub fn tcp_session_receive_handshake() {
        // Init local network
        let filename_to_upload = "venon.jpg";
        let torrent_filename_to_upload = format!("{}.torrent", filename_to_upload);
        let mut tracker_process_child =
            run_tracker().expect("failed to execute tracker process child");

        let seeder_port = SEEDER_TCP_DOWNLOAD_PORT - 1;
        let mut seeder_process_child = run_seeder(&torrent_filename_to_upload, seeder_port)
            .expect("failed to execute seeder process child");
        sleep(Duration::from_secs(1));

        // TCP connection
        let seeder_peer =
            Peer::from_socket_address(SocketAddr::new(IpAddr::V4(SEEDER_IP_ADDRESS), seeder_port));
        let mut tcp_session = match TcpSession::connect(seeder_peer) {
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

        // Try to receive handshake
        // Leecher <-[Handshake]-- Seeder
        sleep(Duration::from_secs(1));
        let received_hanshake_message = match tcp_session.receive() {
            Ok(maybe_message) => match maybe_message {
                Some(message) => message,
                None => {
                    tracker_process_child.kill().unwrap();
                    seeder_process_child.kill().unwrap();
                    panic!("a handshake message was expected")
                }
            },
            Err(error) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("{:?}", error);
            }
        };
        println!("HANDSHAKE received: {:?}\n\n", received_hanshake_message);
        if let Message::Handshake(received_handshake) = received_hanshake_message {
            // Check the handshake corresponding to our torrent has been received
            assert_eq!(received_handshake.info_hash(), info_hash);
        };

        // End of test
        tracker_process_child.kill().unwrap();
        seeder_process_child.kill().unwrap();
    }

    #[ignore = "CI is not setup for integration test using network"]
    #[test]
    pub fn tcp_session_receive_bitfield() {
        // Init local network
        let filename_to_upload = "venon.jpg";
        let torrent_filename_to_upload = format!("{}.torrent", filename_to_upload);
        let mut tracker_process_child =
            run_tracker().expect("failed to execute tracker process child");

        let seeder_port = SEEDER_TCP_DOWNLOAD_PORT - 2;
        let mut seeder_process_child = run_seeder(&torrent_filename_to_upload, seeder_port)
            .expect("failed to execute seeder process child");
        sleep(Duration::from_secs(1));

        // TCP connection
        let seeder_peer =
            Peer::from_socket_address(SocketAddr::new(IpAddr::V4(SEEDER_IP_ADDRESS), seeder_port));
        let mut tcp_session = match TcpSession::connect(seeder_peer) {
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
        let torrent = maybe_torrent.unwrap();
        let info_hash = torrent.info_hash();

        let handshake = Handshake::new(info_hash, PEER_ID);
        let expected_handshake_length_in_byte = 68;
        // Check all the handshake has been sent
        assert_eq!(
            tcp_session.send(handshake).unwrap(),
            expected_handshake_length_in_byte
        );

        // Try to receive handshake
        // Leecher <-[Handshake]-- Seeder
        sleep(Duration::from_secs(1));
        let received_hanshake_message = match tcp_session.receive() {
            Ok(maybe_message) => match maybe_message {
                Some(message) => message,
                None => {
                    tracker_process_child.kill().unwrap();
                    seeder_process_child.kill().unwrap();
                    panic!("a handshake message was expected")
                }
            },
            Err(error) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("{:?}", error);
            }
        };
        if let Message::Handshake(received_handshake) = received_hanshake_message {
            // Check the handshake corresponding to our torrent has been received
            assert_eq!(received_handshake.info_hash(), info_hash);
        };

        // Try to receive Bitfield
        // Leecher <-[Bitfield]-- Seeder
        sleep(Duration::from_secs(1));
        let received_bitfield_message = match tcp_session.receive() {
            Ok(maybe_message) => match maybe_message {
                Some(message) => message,
                None => {
                    tracker_process_child.kill().unwrap();
                    seeder_process_child.kill().unwrap();
                    panic!("a bitfield message was expected")
                }
            },
            Err(error) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("{:?}", error);
            }
        };
        println!("BITFIELD received: {:?}\n\n", received_bitfield_message);
        if let Message::Bitfield(received_bitfield) = received_bitfield_message {
            // Check we received the seeder bitfield full
            let expected_received_bitfield = BitVec::from_bytes(&[
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0x80,
            ]);
            assert_eq!(*received_bitfield.bitfield(), expected_received_bitfield);
        };

        // End of test
        tracker_process_child.kill().unwrap();
        seeder_process_child.kill().unwrap();
    }

    #[ignore = "CI is not setup for integration test using network"]
    #[test]
    pub fn tcp_session_receive_unchoke() {
        // Init local network
        let filename_to_upload = "venon.jpg";
        let torrent_filename_to_upload = format!("{}.torrent", filename_to_upload);
        let mut tracker_process_child =
            run_tracker().expect("failed to execute tracker process child");

        let seeder_port = SEEDER_TCP_DOWNLOAD_PORT - 3;
        let mut seeder_process_child = run_seeder(&torrent_filename_to_upload, seeder_port)
            .expect("failed to execute seeder process child");
        sleep(Duration::from_secs(1));

        // TCP connection
        let seeder_peer =
            Peer::from_socket_address(SocketAddr::new(IpAddr::V4(SEEDER_IP_ADDRESS), seeder_port));
        let mut tcp_session = match TcpSession::connect(seeder_peer) {
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
        let torrent = maybe_torrent.unwrap();
        let info_hash = torrent.info_hash();

        let handshake = Handshake::new(info_hash, PEER_ID);
        let expected_handshake_length_in_byte = 68;
        // Check all the handshake has been sent
        assert_eq!(
            tcp_session.send(handshake).unwrap(),
            expected_handshake_length_in_byte
        );

        // Try to receive handshake
        // Leecher <-[Handshake]-- Seeder
        sleep(Duration::from_secs(1));
        let received_hanshake_message = match tcp_session.receive() {
            Ok(maybe_message) => match maybe_message {
                Some(message) => message,
                None => {
                    tracker_process_child.kill().unwrap();
                    seeder_process_child.kill().unwrap();
                    panic!("a handshake message was expected")
                }
            },
            Err(error) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("{:?}", error);
            }
        };
        if let Message::Handshake(received_handshake) = received_hanshake_message {
            // Check the handshake corresponding to our torrent has been received
            assert_eq!(received_handshake.info_hash(), info_hash);
        };

        // Leecher <-[Bitfield]-- Seeder
        sleep(Duration::from_secs(1));
        let received_bitfield_message = match tcp_session.receive() {
            Ok(maybe_message) => match maybe_message {
                Some(message) => message,
                None => {
                    tracker_process_child.kill().unwrap();
                    seeder_process_child.kill().unwrap();
                    panic!("a bitfield message was expected")
                }
            },
            Err(error) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("{:?}", error);
            }
        };
        if let Message::Bitfield(received_bitfield) = received_bitfield_message {
            // Check we received the seeder bitfield full
            let expected_received_bitfield = BitVec::from_bytes(&[
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0x80,
            ]);
            assert_eq!(*received_bitfield.bitfield(), expected_received_bitfield);
        };

        // Leecher --[Bitfield]-> Seeder
        let empty_bitfield: [u8; 19] = [0; 19];
        let leacher_bitfield = BitVec::from_bytes(&empty_bitfield);
        let bitfield = Bitfield::new(leacher_bitfield);
        let expected_bitfield_message_length = bitfield.message_length() + 4;
        assert_eq!(
            tcp_session.send(bitfield).unwrap() as u32,
            expected_bitfield_message_length
        );

        // Leecher --[Interested]-> Seeder
        let interested = Interested::new();
        let expected_interested_length_in_byte = 5;
        // Check all the interested has been sent
        assert_eq!(
            tcp_session.send(interested).unwrap(),
            expected_interested_length_in_byte
        );

        // Try to receive unchoke
        // Leecher <-[Unchoke]-- Seeder
        sleep(Duration::from_secs(1));
        let received_unchoke_message = match tcp_session.receive() {
            Ok(maybe_message) => match maybe_message {
                Some(message) => message,
                None => {
                    tracker_process_child.kill().unwrap();
                    seeder_process_child.kill().unwrap();
                    panic!("an unchoke message was expected")
                }
            },
            Err(error) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("{:?}", error);
            }
        };
        match received_unchoke_message {
            Message::Unchoke(_) => (),
            _ => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("message enum should be an Unchoke")
            }
        }

        // End of test
        tracker_process_child.kill().unwrap();
        seeder_process_child.kill().unwrap();
    }

    #[ignore = "CI is not setup for integration test using network"]
    #[test]
    pub fn tcp_session_receive_piece() {
        // Init local network
        let filename_to_upload = "venon.jpg";
        let torrent_filename_to_upload = format!("{}.torrent", filename_to_upload);
        let mut tracker_process_child =
            run_tracker().expect("failed to execute tracker process child");

        let seeder_port = SEEDER_TCP_DOWNLOAD_PORT - 4;
        let mut seeder_process_child = run_seeder(&torrent_filename_to_upload, seeder_port)
            .expect("failed to execute seeder process child");
        sleep(Duration::from_secs(1));

        // TCP connection
        let seeder_peer =
            Peer::from_socket_address(SocketAddr::new(IpAddr::V4(SEEDER_IP_ADDRESS), seeder_port));
        let mut tcp_session = match TcpSession::connect(seeder_peer) {
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
        let torrent = maybe_torrent.unwrap();
        let info_hash = torrent.info_hash();

        let handshake = Handshake::new(info_hash, PEER_ID);
        let expected_handshake_length_in_byte = 68;
        // Check all the handshake has been sent
        assert_eq!(
            tcp_session.send(handshake).unwrap(),
            expected_handshake_length_in_byte
        );

        // Try to receive handshake
        // Leecher <-[Handshake]-- Seeder
        sleep(Duration::from_secs(1));
        let received_hanshake_message = match tcp_session.receive() {
            Ok(maybe_message) => match maybe_message {
                Some(message) => message,
                None => {
                    tracker_process_child.kill().unwrap();
                    seeder_process_child.kill().unwrap();
                    panic!("a handshake message was expected")
                }
            },
            Err(error) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("{:?}", error);
            }
        };
        if let Message::Handshake(received_handshake) = received_hanshake_message {
            // Check the handshake corresponding to our torrent has been received
            assert_eq!(received_handshake.info_hash(), info_hash);
        };

        // Leecher <-[Bitfield]-- Seeder
        sleep(Duration::from_secs(1));
        let received_bitfield_message = match tcp_session.receive() {
            Ok(maybe_message) => match maybe_message {
                Some(message) => message,
                None => {
                    tracker_process_child.kill().unwrap();
                    seeder_process_child.kill().unwrap();
                    panic!("a bitfield message was expected")
                }
            },
            Err(error) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("{:?}", error);
            }
        };
        if let Message::Bitfield(received_bitfield) = received_bitfield_message {
            // Check we received the seeder bitfield full
            let expected_received_bitfield = BitVec::from_bytes(&[
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0x80,
            ]);
            assert_eq!(*received_bitfield.bitfield(), expected_received_bitfield);
        };

        // Leecher --[Bitfield]-> Seeder
        let empty_bitfield: [u8; 19] = [0; 19];
        let leacher_bitfield = BitVec::from_bytes(&empty_bitfield);
        let bitfield = Bitfield::new(leacher_bitfield);
        let expected_bitfield_message_length = bitfield.message_length() + 4;
        assert_eq!(
            tcp_session.send(bitfield).unwrap() as u32,
            expected_bitfield_message_length
        );

        // Leecher --[Interested]-> Seeder
        let interested = Interested::new();
        let expected_interested_length_in_bytes = 5;
        // Check all the interested has been sent
        assert_eq!(
            tcp_session.send(interested).unwrap(),
            expected_interested_length_in_bytes
        );

        // Leecher <-[Unchoke]-- Seeder
        sleep(Duration::from_secs(1));
        let received_unchoke_message = match tcp_session.receive() {
            Ok(maybe_message) => match maybe_message {
                Some(message) => message,
                None => {
                    tracker_process_child.kill().unwrap();
                    seeder_process_child.kill().unwrap();
                    panic!("an unchoke message was expected")
                }
            },
            Err(error) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("{:?}", error);
            }
        };
        match received_unchoke_message {
            Message::Unchoke(_) => (),
            _ => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("message enum should be an Unchoke")
            }
        }

        // Leecher --[Request:0]-> Seeder
        let requested_piece_index = 0;
        let requested_begin_offset = 0;
        let requested_block_length = BlockReaderWriter::BIT_TORRENT_BLOCK_SIZE as u32;
        let request = Request::new(
            requested_piece_index,
            requested_begin_offset,
            requested_block_length,
        );
        let expected_request_length_in_bytes = (MessageType::Request.base_length() + 4) as usize;
        // Check all the request has been sent
        assert_eq!(
            tcp_session.send(request).unwrap(),
            expected_request_length_in_bytes
        );

        // Leecher <-[Piece:0] -- Seeder
        sleep(Duration::from_secs(5));
        let received_piece_message = match tcp_session.receive() {
            Ok(maybe_message) => match maybe_message {
                Some(message) => message,
                None => {
                    tracker_process_child.kill().unwrap();
                    seeder_process_child.kill().unwrap();
                    panic!("an piece message was expected")
                }
            },
            Err(error) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("{:?}", error);
            }
        };

        let received_piece = match received_piece_message {
            Message::Piece(piece) => piece,
            _ => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("message enum should be an Piece")
            }
        };
        assert_eq!(received_piece.piece_index(), requested_piece_index);
        assert_eq!(
            received_piece.begin_offset_of_piece(),
            requested_begin_offset
        );
        let expected_piece_bytes =
            read_bytes_from(&path_build_to_pwp_message("venon_piece_0x00_0x0000.bin"));
        assert_eq!(received_piece.data().len(), (&expected_piece_bytes).len());
        assert_eq!(received_piece.data(), &expected_piece_bytes);

        // Close peer and tracker
        tracker_process_child.kill().unwrap();
        seeder_process_child.kill().unwrap();
    }

    #[ignore = "CI is not setup for integration test using network"]
    #[test]
    pub fn tcp_session_read_empty_socket() {
        // Init local network (tracker and seeder)
        let filename_to_upload = "venon.jpg";
        let torrent_filename_to_upload = format!("{}.torrent", filename_to_upload);
        let mut tracker_process_child =
            run_tracker().expect("failed to execute tracker process child");

        let seeder_port = SEEDER_TCP_DOWNLOAD_PORT - 5;
        let mut seeder_process_child = run_seeder(&torrent_filename_to_upload, seeder_port)
            .expect("failed to execute seeder process child");
        sleep(Duration::from_secs(5));

        // TCP connection
        let seeder_peer =
            Peer::from_socket_address(SocketAddr::new(IpAddr::V4(SEEDER_IP_ADDRESS), seeder_port));
        let mut tcp_session = match TcpSession::connect(seeder_peer) {
            Ok(session) => session,
            Err(_) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("cannot connect to seeder");
            }
        };

        // Try to receive data when we don't expect data to be received
        match tcp_session.receive() {
            Ok(maybe_message) => match maybe_message {
                Some(_) => {
                    tracker_process_child.kill().unwrap();
                    seeder_process_child.kill().unwrap();
                    panic!("no message should have been received")
                }
                None => {
                    tracker_process_child.kill().unwrap();
                    seeder_process_child.kill().unwrap();
                    assert!(true)
                }
            },
            Err(error) => {
                tracker_process_child.kill().unwrap();
                seeder_process_child.kill().unwrap();
                panic!("{:?}", error);
            }
        };
    }
}
