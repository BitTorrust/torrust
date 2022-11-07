use crate::pwp::MessageType;

#[derive(Debug)]
pub struct Have {
    message_length: u32,
    message_type: u8,
    piece_index: u32,
}

impl Have {
    pub fn new(piece_index: u32) -> Self {
        Self {
            message_length: MessageType::Have.base_length(),
            message_type: MessageType::Have.into_u8(),
            piece_index,
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut serialized_message: Vec<u8> = Vec::new();
        serialized_message.extend(self.message_length.to_be_bytes());
        serialized_message.push(self.message_type);
        serialized_message.extend(self.piece_index.to_be_bytes());
        serialized_message
    }
}
