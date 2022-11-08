#[cfg(test)]
pub mod unittest {
    use crate::pwp::{
        Bitfield, Have, Interested, IntoBytes, NotIterested, Piece, Request, Unchoke,
    };
    use bit_vec::BitVec;
    use std::{fs::File, io::Read, path::Path};

    fn read_bytes_from(pathfile: &str) -> Vec<u8> {
        let data_filepath = Path::new(pathfile);
        let mut data_file = File::open(data_filepath).unwrap();
        let mut data_buffer: Vec<u8> = Vec::new();
        data_file.read_to_end(&mut data_buffer).unwrap();
        data_buffer
    }

    #[test]
    pub fn unchoke_message_into_bytes() {
        let expected = read_bytes_from(
            "samples/peer_wire_protocol-messages/expected_unchoke_bytes_in_hex.bin",
        );
        let unchoke_message = Unchoke::new();

        assert_eq!(unchoke_message.into_bytes(), expected);
    }

    #[test]
    pub fn interested_message_into_bytes() {
        let expected = read_bytes_from(
            "samples/peer_wire_protocol-messages/expected_interested_bytes_in_hex.bin",
        );
        let interested_message = Interested::new();

        assert_eq!(interested_message.into_bytes(), expected);
    }

    #[test]
    pub fn bitfield_message_into_bytes() {
        let expected = read_bytes_from(
            "samples/peer_wire_protocol-messages/expected_bitfield_bytes_in_hex.bin",
        );

        let bitfield = BitVec::from_bytes(&[0xff, 0xe0]);
        let bitfield_message = Bitfield::new(bitfield);

        assert_eq!(bitfield_message.into_bytes(), expected);
    }

    #[test]
    pub fn piece_message_into_bytes() {
        // Init Piece struct
        let data =
            read_bytes_from("samples/peer_wire_protocol-messages/data_pieces_bytes_in_hex.bin");
        let piece_message = Piece::new(6, 0, data);

        let expected_bytes =
            read_bytes_from("samples/peer_wire_protocol-messages/expected_pieces_bytes_in_hex.bin");

        assert_eq!(piece_message.into_bytes(), expected_bytes);
    }

    #[test]
    pub fn request_message_into_bytes() {
        let request_message = Request::new(6, 0, 0x4000);

        let expected_bytes = read_bytes_from(
            "samples/peer_wire_protocol-messages/expected_request_bytes_in_hex.bin",
        );

        assert_eq!(request_message.into_bytes(), expected_bytes);
    }

    #[test]
    pub fn not_interested_message_into_bytes() {
        let not_interested_message = NotIterested::new();

        let expected_bytes = read_bytes_from(
            "samples/peer_wire_protocol-messages/expected_not_interested_bytes_in_hex.bin",
        );

        assert_eq!(not_interested_message.into_bytes(), expected_bytes);
    }

    #[test]
    pub fn have_message_into_bytes() {
        let have_message = Have::new(0x1);

        let expected_bytes =
            read_bytes_from("samples/peer_wire_protocol-messages/expected_have_bytes_in_hex.bin");

        assert_eq!(have_message.into_bytes(), expected_bytes);
    }
}
