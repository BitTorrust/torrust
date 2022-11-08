use crate::pwp::{IntoBytes, MessageType};

#[derive(Debug)]
pub struct Piece {
    message_length: u32,
    message_type: u8,
    /// integer specifying the zero-based piece index
    piece_index: u32,
    /// integer specifying the zero-based byte offset within the piece
    begin_offset_of_piece: u32,
    /// block of data, which is a subset of the piece specified by index
    data: Vec<u8>,
}

impl Piece {
    pub fn new(piece_index: u32, begin_offset_of_piece: u32, data: Vec<u8>) -> Self {
        Self {
            message_length: MessageType::Piece.base_length() + data.len() as u32,
            message_type: MessageType::Piece.id(),
            piece_index,
            begin_offset_of_piece,
            data,
        }
    }
}

impl IntoBytes for Piece {
    fn into_bytes(self) -> Vec<u8> {
        let mut serialized_message: Vec<u8> = Vec::new();
        serialized_message.extend(self.message_length.to_be_bytes());
        serialized_message.push(self.message_type);
        serialized_message.extend(self.piece_index.to_be_bytes());
        serialized_message.extend(self.begin_offset_of_piece.to_be_bytes());
        serialized_message.extend(self.data);
        serialized_message
    }
}
