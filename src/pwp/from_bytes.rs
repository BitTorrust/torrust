use crate::Error;

pub const PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES: u32 = 4;

pub trait FromBytes {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;
}
