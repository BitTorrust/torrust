#[cfg(test)]
pub mod pwp {
    //TODO test pieces, then implement request into_bytes and test it
    use crate::pwp::{Piece, Request};
    use std::{fs::File, io::Read, path::Path};
    #[test]
    pub fn format_piece_message_into_bytes() {
        let data_filepath =
            Path::new("samples/peer_wire_protocol-messages/data_pieces_bytes_in_hex.bin");
        let mut data_file = File::open(data_filepath).unwrap();
        let mut data_buffer: Vec<u8> = Vec::new();
        data_file.read_to_end(&mut data_buffer).unwrap();

        let piece_message = Piece::new(6, 0, data_buffer);

        let message_filepath =
            Path::new("samples/peer_wire_protocol-messages/expected_pieces_bytes_in_hex.bin");
        let mut message_file = File::open(message_filepath).unwrap();
        let mut message_buffer: Vec<u8> = Vec::new();
        message_file.read_to_end(&mut message_buffer).unwrap();
        assert_eq!(piece_message.into_bytes(), message_buffer,);
    }
}
