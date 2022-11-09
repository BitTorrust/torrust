use crate::{
    pwp::{from_bytes, FromBytes, IntoBytes, MandatoryBitTorrentMessageFields, MessageType},
    Error,
};
use bit_vec::BitVec;
use sha1::digest::typenum::Bit;

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

    pub fn bitfield(&self) -> &BitVec {
        &self.bitfield
    }
}

impl MandatoryBitTorrentMessageFields for Bitfield {
    fn message_length(&self) -> u32 {
        self.message_length
    }

    fn message_type(&self) -> u8 {
        self.message_type
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

impl FromBytes for Bitfield {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), Error> {
        println!("bytes {:?}", bytes);
        if (bytes.len() as u32)
            < MessageType::Bitfield.base_length()
                + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES
        {
            return Err(Error::BytesArrayTooShort);
        }

        let message_length = u32::from_be_bytes(
            bytes[0..4]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentMessageLength)?,
        );

        let message_type = bytes[4];
        if message_type != MessageType::Bitfield.id() {
            return Err(Error::MessageTypeDoesNotMatchWithExpectedOne);
        }

        let bitfield_end_offset = (5 + message_length - 1) as usize;
        let bitfield = BitVec::from_bytes(&bytes[5..bitfield_end_offset]);

        Ok((
            Self {
                message_length,
                message_type,
                bitfield,
            },
            (message_length + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES) as usize,
        ))
    }
}
