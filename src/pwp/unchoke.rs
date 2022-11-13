use crate::pwp::{from_bytes, FromBytes, IntoBytes, MandatoryBitTorrentMessageFields, MessageType};
use crate::Error;

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

impl MandatoryBitTorrentMessageFields for Unchoke {
    fn message_length(&self) -> u32 {
        self.message_length
    }

    fn message_type(&self) -> u8 {
        self.message_type
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

impl FromBytes for Unchoke {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), Error> {
        if (bytes.len() as u32)
            < MessageType::Unchoke.base_length()
                + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES
        {
            return Err(Error::BytesArrayTooShort);
        }

        let message_length = u32::from_be_bytes(
            bytes[0..4]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentMessageLength)?,
        );
        if message_length != MessageType::Unchoke.base_length() {
            return Err(Error::MessageLengthDoesNotMatchWithExpectedOne);
        }

        let message_type = bytes[4];
        if message_type != MessageType::Unchoke.id() {
            return Err(Error::MessageTypeDoesNotMatchWithExpectedOne);
        }

        Ok((
            Self {
                message_length,
                message_type,
            },
            (message_length + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES) as usize,
        ))
    }
}
