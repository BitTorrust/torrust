use crate::pwp::MessageType;

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
