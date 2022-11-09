pub enum MessageType {
    Unchoke,
    Interested,
    NotInterested,
    Have,
    Bitfield,
    Request,
    Piece,
}

// Documentation for message: https://wiki.theory.org/BitTorrentSpecification#Messages
impl MessageType {
    pub fn id(self) -> u8 {
        match self {
            MessageType::Unchoke => 1,
            MessageType::Interested => 2,
            MessageType::NotInterested => 3,
            MessageType::Have => 4,
            MessageType::Bitfield => 5,
            MessageType::Request => 6,
            MessageType::Piece => 7,
        }
    }

    /// Length of the message without variable size field and length field (4 bytes) taken in account
    pub fn base_length(self) -> u32 {
        match self {
            MessageType::Unchoke => 1,
            MessageType::Interested => 1,
            MessageType::NotInterested => 1,
            MessageType::Have => 5,
            MessageType::Bitfield => 1,
            MessageType::Request => 13,
            MessageType::Piece => 9,
        }
    }
}
