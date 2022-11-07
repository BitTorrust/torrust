pub enum MessageType {
    NotIterested,
    Have,
    Request,
    Piece,
}

impl MessageType {
    pub fn into_u8(self) -> u8 {
        match self {
            MessageType::NotIterested => 3,
            MessageType::Have => 4,
            MessageType::Request => 6,
            MessageType::Piece => 7,
        }
    }

    /// Length of the message without variable size field taken in account
    pub fn base_length(self) -> u32 {
        match self {
            MessageType::NotIterested => 1,
            MessageType::Have => 5,
            MessageType::Request => 13,
            MessageType::Piece => 9,
        }
    }
}
