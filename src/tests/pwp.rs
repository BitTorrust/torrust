#[cfg(test)]
pub mod unittest {
    use crate::pwp::{
        Bitfield, FromBytes, Handshake, Have, Interested, IntoBytes, NotIterested, Piece, Request,
        Unchoke,
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
        let expected = read_bytes_from(&path_builder("handshake.bin"));
        let handshake_bytes = Handshake::new(INFO_ID, PEER_ID);

        assert_eq!(handshake_bytes.into_bytes(), expected);
    }

    #[test]
    pub fn unchoke_message_into_bytes() {
        let expected = read_bytes_from(&path_builder("unchoke.bin"));
        let unchoke_message = Unchoke::new();

        assert_eq!(unchoke_message.into_bytes(), expected);
    }

    #[test]
    pub fn interested_message_into_bytes() {
        let expected = read_bytes_from(&path_builder("interested.bin"));
        let interested_message = Interested::new();

        assert_eq!(interested_message.into_bytes(), expected);
    }

    #[test]
    pub fn bitfield_message_into_bytes() {
        let expected = read_bytes_from(&path_builder("bitfield.bin"));
        let bitfield_message = Bitfield::new(BitVec::from_bytes(&[0xff, 0xe0]));

        assert_eq!(bitfield_message.into_bytes(), expected);
    }

    #[test]
    pub fn piece_message_into_bytes() {
        let data = read_bytes_from(&path_builder("pieces_data.bin"));
        let piece_message_to_test = Piece::new(6, 0, data);
        let expected_bytes = read_bytes_from(&path_builder("pieces.bin"));

        assert_eq!(piece_message_to_test.into_bytes(), expected_bytes);
    }

    #[test]
    pub fn request_message_into_bytes() {
        let request_message_to_test = Request::new(6, 0, 0x4000);
        let expected_bytes = read_bytes_from(&path_builder("request.bin"));

        assert_eq!(request_message_to_test.into_bytes(), expected_bytes);
    }

    #[test]
    pub fn request_message_from_bytes() {
        let request_bytes = read_bytes_from(&path_builder("request.bin"));
        let request_message = Request::from_bytes(&request_bytes).unwrap();

        let expected_request = Request::new(6, 0, 0x4000);

        assert_eq!(request_message, expected_request);
    }

    #[test]
    pub fn not_interested_message_into_bytes() {
        let not_interested_message = NotIterested::new();
        let expected_bytes = read_bytes_from(&path_builder("not_interested.bin"));

        assert_eq!(not_interested_message.into_bytes(), expected_bytes);
    }

    #[test]
    pub fn have_message_into_bytes() {
        let have_message = Have::new(0x1);
        let expected_bytes = read_bytes_from(&path_builder("have.bin"));

        assert_eq!(have_message.into_bytes(), expected_bytes);
    }
}
