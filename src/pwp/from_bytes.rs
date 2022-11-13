use crate::pwp::MessageType;
use crate::Error;
use strum::IntoEnumIterator; // 0.17.1

pub const PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES: u32 = 4;
pub const PWP_MESSAGE_TYPE_FIELD_OFFSET_IN_BYTES: usize = 4;

pub trait FromBytes {
    // Returns the structure and the number of bytes used for generating the struct of the bytes array
    fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), Error>
    where
        Self: Sized + FromBytes;
}

pub fn identity_first_message_type_of(bytes: &[u8]) -> Result<MessageType, Error> {
    for enum_instance in MessageType::iter() {
        if bytes[PWP_MESSAGE_TYPE_FIELD_OFFSET_IN_BYTES] == enum_instance.id() {
            return Ok(enum_instance);
        }
    }
    Err(Error::FailedToFindTheMessageTypeOfRawBytes)
}
