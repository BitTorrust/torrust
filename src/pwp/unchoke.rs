use crate::pwp::{IntoBytes, MessageType};

/// unchoke: <len=0001><id=1>
#[derive(Debug)]
pub struct Unchoke {
    message_length: u32,
    message_type: u8,
}

impl Unchoke {
    pub fn new() -> Self {
        Self {
            message_length: MessageType::Unchoke.base_length(),
            message_type: MessageType::Unchoke.id(),
        }
    }
}

impl IntoBytes for Unchoke {
    fn into_bytes(self) -> Vec<u8> {
        let mut serialized_message = Vec::new();

        serialized_message.extend(self.message_length.to_be_bytes());
        serialized_message.push(self.message_type);

        serialized_message
    }
}
