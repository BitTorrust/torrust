use crate::pwp::{from_bytes, FromBytes, IntoBytes, MandatoryBitTorrentMessageFields, MessageType};
use crate::Error;

#[derive(Debug)]
pub struct Have {
    message_length: u32,
    message_type: u8,
    piece_index: u32,
}

impl Have {
    pub fn new(piece_index: u32) -> Self {
        Self {
            message_length: MessageType::Have.base_length(),
            message_type: MessageType::Have.id(),
            piece_index,
        }
    }

    pub fn have(&self) -> u32 {
        self.piece_index
    }
}

impl MandatoryBitTorrentMessageFields for Have {
    fn message_length(&self) -> u32 {
        self.message_length
    }

    fn message_type(&self) -> u8 {
        self.message_type
    }
}

impl IntoBytes for Have {
    fn into_bytes(self) -> Vec<u8> {
        let mut serialized_message: Vec<u8> = Vec::new();
        serialized_message.extend(self.message_length.to_be_bytes());
        serialized_message.push(self.message_type);
        serialized_message.extend(self.piece_index.to_be_bytes());
        serialized_message
    }
}

impl FromBytes for Have {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), Error> {
        if (bytes.len() as u32)
            < (MessageType::Have.base_length() + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES)
        {
            return Err(Error::BytesArrayTooShort);
        }

        let message_length = u32::from_be_bytes(
            bytes[0..4]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentMessageLength)?,
        );
        if message_length != MessageType::Have.base_length() {
            return Err(Error::MessageLengthDoesNotMatchWithExpectedOne);
        }

        let message_type = bytes[4];
        if message_type != MessageType::Have.id() {
            return Err(Error::MessageTypeDoesNotMatchWithExpectedOne);
        }

        let piece_index = u32::from_be_bytes(
            bytes[13..17]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentHaveMessagePieceIndex)?,
        );

        Ok((
            Self {
                message_length,
                message_type,
                piece_index,
            },
            (message_length + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES) as usize,
        ))
    }
}
