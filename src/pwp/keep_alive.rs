use crate::{
    from_bytes, Error, FromBytes, IntoBytes, MandatoryBitTorrentMessageFields, MessageType,
};

// keep-alive: <len=0000>
#[derive(Debug)]
pub struct KeepAlive {
    message_length: u32,
    message_type: u8,
}

impl KeepAlive {
    pub fn new() -> KeepAlive {
        KeepAlive {
            message_length: MessageType::KeepAlive.base_length(),
            message_type: MessageType::KeepAlive.id(),
        }
    }
}

impl MandatoryBitTorrentMessageFields for KeepAlive {
    fn message_length(&self) -> u32 {
        self.message_length
    }

    fn message_type(&self) -> u8 {
        self.message_type
    }
}

impl IntoBytes for KeepAlive {
    fn into_bytes(self) -> Vec<u8> {
        let mut serialized_message = Vec::new();

        serialized_message.extend(self.message_length.to_be_bytes());

        serialized_message
    }
}

impl FromBytes for KeepAlive {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), Error> {
        println!("bytes.len() {}", bytes.len());
        if (bytes.len() as u32)
            < MessageType::KeepAlive.base_length()
                + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES
        {
            return Err(Error::BytesArrayTooShort);
        }

        let message_length = u32::from_be_bytes(
            bytes[0..4]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentMessageLength)?,
        );
        if message_length != MessageType::KeepAlive.base_length() {
            return Err(Error::MessageLengthDoesNotMatchWithExpectedOne);
        }

        Ok((
            Self {
                message_length,
                message_type: MessageType::KeepAlive.id(),
            },
            (message_length + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES) as usize,
        ))
    }
}
