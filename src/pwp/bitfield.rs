use crate::pwp::{IntoBytes, MessageType};
use bit_vec::BitVec;

/// bitfield: <len=0001+X><id=5><bitfield>
#[derive(Debug)]
pub struct Bitfield {
    message_length: u32,
    message_type: u8,
    bitfield: BitVec,
}

impl Bitfield {
    pub fn new(bitfield: BitVec) -> Self {
        let bitfield_len = bitfield.to_bytes().len() as u32;

        Self {
            message_length: MessageType::Bitfield.base_length() + bitfield_len,
            message_type: MessageType::Bitfield.id(),
            bitfield,
        }
    }
}

impl IntoBytes for Bitfield {
    fn into_bytes(self) -> Vec<u8> {
        let mut serialized_message = Vec::new();
        let bitfield = self.bitfield.to_bytes();

        serialized_message.extend(self.message_length.to_be_bytes());
        serialized_message.push(self.message_type);
        serialized_message.extend(bitfield);

        serialized_message
    }
}
