use crate::Error;

pub const PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES: u32 = 4;

pub trait FromBytes {
    // Returns the structure and the number of bytes used for generating the struct of the bytes array
    fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), Error>
    where
        Self: Sized;
}
