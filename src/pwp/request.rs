use crate::pwp::{IntoBytes, MessageType};

#[derive(Debug)]
pub struct Request {
    message_length: u32,
    message_type: u8,
    /// integer specifying the zero-based piece index
    piece_index: u32,
    /// integer specifying the zero-based byte offset within the piece
    begin_offset: u32,
    /// integer specifying the requested length
    piece_length: u32,
}

impl Request {
    pub fn new(piece_index: u32, begin_offset: u32, piece_length: u32) -> Self {
        Self {
            message_length: MessageType::Request.base_length(),
            message_type: MessageType::Request.into_u8(),
            piece_index,
            begin_offset,
            piece_length,
        }
    }
}

impl IntoBytes for Request {
    fn into_bytes(self) -> Vec<u8> {
        let mut serialized_message: Vec<u8> = Vec::new();
        serialized_message.extend(self.message_length.to_be_bytes());
        serialized_message.push(self.message_type);
        serialized_message.extend(self.piece_index.to_be_bytes());
        serialized_message.extend(self.begin_offset.to_be_bytes());
        serialized_message.extend(self.piece_length.to_be_bytes());
        serialized_message
    }
}
