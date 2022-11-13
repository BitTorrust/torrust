#[cfg(test)]
pub mod unittest {
    use crate::pwp::{
        Bitfield, Handshake, Have, Interested, IntoBytes, NotIterested, Piece, Request, Unchoke,
    };
    use bit_vec::BitVec;
    use std::{fs::File, io::Read, path::Path};

    const INFO_ID: [u8; 20] = [
        0x06, 0x71, 0x33, 0xac, 0xe5, 0xdd, 0x0c, 0x50, 0x27, 0xb9, 0x9d, 0xe5, 0xd4, 0xba, 0x51,
        0x28, 0x28, 0x20, 0x8d, 0x5b,
    ];

    const PEER_ID: [u8; 20] = [
        0x2d, 0x42, 0x45, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x6e, 0x9a, 0xb4, 0x40, 0x2c, 0x62, 0x2e,
        0x2e, 0x7a, 0x71, 0x5d, 0x9d,
    ];

    pub fn read_bytes_from(pathfile: &str) -> Vec<u8> {
        let data_filepath = Path::new(pathfile);
        let mut data_file = File::open(data_filepath).unwrap();
        let mut data_buffer: Vec<u8> = Vec::new();
        data_file.read_to_end(&mut data_buffer).unwrap();
        data_buffer
    }

    pub fn path_builder(file: &str) -> String {
        "samples/peer_wire_protocol-messages/".to_string() + file
    }

    #[test]
    pub fn handshake_message_into_bytes() {
        let expected = read_bytes_from(&path_builder("expected_handshake_bytes_in_hex.bin"));
        let handshake_bytes = Handshake::new(INFO_ID, PEER_ID);

        assert_eq!(handshake_bytes.into_bytes(), expected);
    }

    #[test]
    pub fn unchoke_message_into_bytes() {
        let expected = read_bytes_from(&path_builder("expected_unchoke_bytes_in_hex.bin"));
        let unchoke_message = Unchoke::new();

        assert_eq!(unchoke_message.into_bytes(), expected);
    }

    #[test]
    pub fn interested_message_into_bytes() {
        let expected = read_bytes_from(&path_builder("expected_interested_bytes_in_hex.bin"));
        let interested_message = Interested::new();

        assert_eq!(interested_message.into_bytes(), expected);
    }

    #[test]
    pub fn bitfield_message_into_bytes() {
        let expected = read_bytes_from(&path_builder("expected_bitfield_bytes_in_hex.bin"));
        let bitfield_message = Bitfield::new(BitVec::from_bytes(&[0xff, 0xe0]));

        assert_eq!(bitfield_message.into_bytes(), expected);
    }

    #[test]
    pub fn piece_message_into_bytes() {
        let data = read_bytes_from(&path_builder("data_pieces_bytes_in_hex.bin"));
        let piece_message = Piece::new(6, 0, data);
        let expected_bytes = read_bytes_from(&path_builder("expected_pieces_bytes_in_hex.bin"));

        assert_eq!(piece_message.into_bytes(), expected_bytes);
    }

    #[test]
    pub fn request_message_into_bytes() {
        let request_message = Request::new(6, 0, 0x4000);
        let expected_bytes = read_bytes_from(&path_builder("expected_request_bytes_in_hex.bin"));

        assert_eq!(request_message.into_bytes(), expected_bytes);
    }

    #[test]
    pub fn not_interested_message_into_bytes() {
        let not_interested_message = NotIterested::new();
        let expected_bytes =
            read_bytes_from(&path_builder("expected_not_interested_bytes_in_hex.bin"));

        assert_eq!(not_interested_message.into_bytes(), expected_bytes);
    }

    #[test]
    pub fn have_message_into_bytes() {
        let have_message = Have::new(0x1);
        let expected_bytes = read_bytes_from(&path_builder("expected_have_bytes_in_hex.bin"));

        assert_eq!(have_message.into_bytes(), expected_bytes);
    }
}
