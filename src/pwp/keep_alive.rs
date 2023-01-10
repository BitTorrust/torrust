use crate::{MandatoryBitTorrentMessageFields, IntoBytes, FromBytes, Error, MessageType, from_bytes};


// interested: <len=0001><id=2>
#[derive(Debug)]
pub struct KeepAlive {
    message_length: u32,
    message_type: u8,
}

impl KeepAlive {
    const FALSE_KEEPALIVE_ID : u8 = 255;
    pub fn new() -> KeepAlive {
        KeepAlive {
            message_length: MessageType::KeepAlive.base_length(),
            message_type: KeepAlive::FALSE_KEEPALIVE_ID,
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
        if message_length != MessageType::Interested.base_length() {
            return Err(Error::MessageLengthDoesNotMatchWithExpectedOne);
        }

        Ok((
            Self {
                message_length,
                message_type: KeepAlive::FALSE_KEEPALIVE_ID,
            },
            (message_length + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES) as usize,
        ))
    }
}
