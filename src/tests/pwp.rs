#[cfg(test)]
pub mod unittest {
    use crate::pwp::{Have, IntoBytes, NotIterested, Piece, Request};
    use std::{fs::File, io::Read, path::Path};
    #[test]
    pub fn piece_message_into_bytes() {
        // Init Piece struct
        let data_filepath =
            Path::new("samples/peer_wire_protocol-messages/data_pieces_bytes_in_hex.bin");
        let mut data_file = File::open(data_filepath).unwrap();
        let mut data_buffer: Vec<u8> = Vec::new();
        data_file.read_to_end(&mut data_buffer).unwrap();
        let piece_message = Piece::new(6, 0, data_buffer);

        // Expected bytes
        let message_filepath =
            Path::new("samples/peer_wire_protocol-messages/expected_pieces_bytes_in_hex.bin");
        let mut message_file = File::open(message_filepath).unwrap();
        let mut message_buffer: Vec<u8> = Vec::new();
        message_file.read_to_end(&mut message_buffer).unwrap();

        assert_eq!(piece_message.into_bytes(), message_buffer,);
    }

    #[test]
    pub fn request_message_into_bytes() {
        // Init Request struct
        let request_message = Request::new(6, 0, 0x4000);

        // Expected bytes
        let message_filepath =
            Path::new("samples/peer_wire_protocol-messages/expected_request_bytes_in_hex.bin");
        let mut message_file = File::open(message_filepath).unwrap();
        let mut message_buffer: Vec<u8> = Vec::new();
        message_file.read_to_end(&mut message_buffer).unwrap();

        assert_eq!(request_message.into_bytes(), message_buffer,);
    }

    #[test]
    pub fn not_interested_message_into_bytes() {
        // Init NotIterested struct
        let not_interested_message = NotIterested::new();

        // Expected bytes
        let message_filepath = Path::new(
            "samples/peer_wire_protocol-messages/expected_not_interested_bytes_in_hex.bin",
        );
        let mut message_file = File::open(message_filepath).unwrap();
        let mut message_buffer: Vec<u8> = Vec::new();
        message_file.read_to_end(&mut message_buffer).unwrap();

        assert_eq!(not_interested_message.into_bytes(), message_buffer,);
    }

    #[test]
    pub fn have_message_into_bytes() {
        // Init Have struct
        let have_message = Have::new(0x1);

        // Expected bytes
        let message_filepath =
            Path::new("samples/peer_wire_protocol-messages/expected_have_bytes_in_hex.bin");
        let mut message_file = File::open(message_filepath).unwrap();
        let mut message_buffer: Vec<u8> = Vec::new();
        message_file.read_to_end(&mut message_buffer).unwrap();

        assert_eq!(have_message.into_bytes(), message_buffer,);
    }
}
