use crate::pwp::{IntoBytes, MessageType};

// interested: <len=0001><id=2>
#[derive(Debug)]
pub struct Interested {
    message_length: u32,
    message_type: u8,
}

impl Interested {
    pub fn new() -> Self {
        Self {
            message_length: MessageType::Interested.base_length(),
            message_type: MessageType::Interested.id(),
        }
    }
}

impl IntoBytes for Interested {
    fn into_bytes(self) -> Vec<u8> {
        let mut serialized_message = Vec::new();

        serialized_message.extend(self.message_length.to_be_bytes());
        serialized_message.push(self.message_type);

        serialized_message
    }
}
