pub enum MessageType {
    Request,
    Piece,
}

impl MessageType {
    pub fn into_u8(self) -> u8 {
        match self {
            MessageType::Request => 5,
            MessageType::Piece => 7,
        }
    }

    /// Length of the message without variable size field taken in account
    pub fn base_length(self) -> u32{
        match self {
            MessageType::Request => 13,
            MessageType::Piece => 9,
        }
    }
}
