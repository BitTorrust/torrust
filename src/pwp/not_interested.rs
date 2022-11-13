use crate::{
    pwp::{from_bytes, FromBytes, IntoBytes, MandatoryBitTorrentMessageFields, MessageType},
    Error,
};

#[derive(Debug)]
pub struct NotInterested {
    message_length: u32,
    message_type: u8,
}

impl NotInterested {
    pub fn new() -> Self {
        Self {
            message_length: MessageType::NotInterested.base_length(),
            message_type: MessageType::NotInterested.id(),
        }
    }
}

impl MandatoryBitTorrentMessageFields for NotInterested {
    fn message_length(&self) -> u32 {
        self.message_length
    }

    fn message_type(&self) -> u8 {
        self.message_type
    }
}

impl IntoBytes for NotInterested {
    fn into_bytes(self) -> Vec<u8> {
        let mut serialized_message: Vec<u8> = Vec::new();
        serialized_message.extend(self.message_length.to_be_bytes());
        serialized_message.push(self.message_type);
        serialized_message
    }
}

impl FromBytes for NotInterested {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), Error> {
        if (bytes.len() as u32)
            < MessageType::NotInterested.base_length()
                + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES
        {
            return Err(Error::BytesArrayTooShort);
        }

        let message_length = u32::from_be_bytes(
            bytes[0..4]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentMessageLength)?,
        );
        if message_length != MessageType::NotInterested.base_length() {
            return Err(Error::MessageLengthDoesNotMatchWithExpectedOne);
        }

        let message_type = bytes[4];
        if message_type != MessageType::NotInterested.id() {
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
