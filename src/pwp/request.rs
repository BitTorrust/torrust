use crate::pwp::{from_bytes, FromBytes, IntoBytes, MessageType};
use crate::Error;

/// request: <len=0013><id=6><index><begin><length>
#[derive(Debug, Copy, Clone)]
pub struct Request {
    message_length: u32,
    message_type: u8,
    /// integer specifying the zero-based piece index
    piece_index: u32,
    /// integer specifying the zero-based byte offset within the piece
    begin_offset: u32,
    /// integer specifying the requested length
    piece_length: u32,
}

impl Request {
    pub fn new(piece_index: u32, begin_offset: u32, piece_length: u32) -> Self {
        Self {
            message_length: MessageType::Request.base_length(),
            message_type: MessageType::Request.id(),
            piece_index,
            begin_offset,
            piece_length,
        }
    }

    pub fn message_length(&self) -> u32 {
        self.message_length
    }

    pub fn message_type(&self) -> u8 {
        self.message_type
    }

    pub fn piece_index(&self) -> u32 {
        self.piece_index
    }

    pub fn begin_offset(&self) -> u32 {
        self.begin_offset
    }

    pub fn piece_length(&self) -> u32 {
        self.piece_length
    }
}

impl IntoBytes for Request {
    fn into_bytes(self) -> Vec<u8> {
        let mut serialized_message: Vec<u8> = Vec::new();
        serialized_message.extend(self.message_length.to_be_bytes());
        serialized_message.push(self.message_type);
        serialized_message.extend(self.piece_index.to_be_bytes());
        serialized_message.extend(self.begin_offset.to_be_bytes());
        serialized_message.extend(self.piece_length.to_be_bytes());
        serialized_message
    }
}

impl FromBytes for Request {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() as u32
            != MessageType::Request.base_length()
                + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES
        {
            return Err(Error::BytesArrayTooShort);
        }

        let message_length = u32::from_be_bytes(
            bytes[0..4]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentMessageLength)?,
        );
        if message_length != MessageType::Request.base_length() {
            return Err(Error::MessageLengthDoesNotMatchWithExpectedOne);
        }

        let message_type = bytes[4];
        if message_type != MessageType::Request.id() {
            return Err(Error::MessageTypeDoesNotMatchWithExpectedOne);
        }

        let piece_index = u32::from_be_bytes(
            bytes[5..9]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentRequestMessagePieceIndex)?,
        );

        let begin_offset = u32::from_be_bytes(
            bytes[9..13]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentRequestMessageBeginOffset)?,
        );

        let piece_length = u32::from_be_bytes(
            bytes[13..17]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentRequestMessagePieceLength)?,
        );

        Ok(Self {
            message_length,
            message_type,
            piece_index,
            begin_offset,
            piece_length,
        })
    }
}
