use crate::pwp::{IntoBytes, MessageType};

#[derive(Debug)]
pub struct NotIterested {
    message_length: u32,
    message_type: u8,
}

impl NotIterested {
    pub fn new() -> Self {
        Self {
            message_length: MessageType::NotIterested.base_length(),
            message_type: MessageType::NotIterested.id(),
        }
    }
}

impl IntoBytes for NotIterested {
    fn into_bytes(self) -> Vec<u8> {
        let mut serialized_message: Vec<u8> = Vec::new();
        serialized_message.extend(self.message_length.to_be_bytes());
        serialized_message.push(self.message_type);
        serialized_message
    }
}
