enum MessageType {
    Request,
}

impl MessageType {
    pub fn into_u8(self) -> u8 {
        match self {
            MessageType::Request => 5,
        }
    }
}
