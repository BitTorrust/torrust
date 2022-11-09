use crate::pwp::{from_bytes, FromBytes, IntoBytes, MandatoryBitTorrentMessageFields, MessageType};
use crate::Error;

#[derive(Debug)]
pub struct Piece {
    message_length: u32,
    message_type: u8,
    /// integer specifying the zero-based piece index
    piece_index: u32,
    /// integer specifying the zero-based byte offset within the piece
    begin_offset_of_piece: u32,
    /// block of data, which is a subset of the piece specified by index
    data: Vec<u8>,
}

impl Piece {
    pub fn new(piece_index: u32, begin_offset_of_piece: u32, data: Vec<u8>) -> Self {
        Self {
            message_length: MessageType::Piece.base_length() + data.len() as u32,
            message_type: MessageType::Piece.id(),
            piece_index,
            begin_offset_of_piece,
            data,
        }
    }

    pub fn piece_index(&self) -> u32 {
        self.piece_index
    }

    pub fn begin_offset_of_piece(&self) -> u32 {
        self.begin_offset_of_piece
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}

impl MandatoryBitTorrentMessageFields for Piece {
    fn message_length(&self) -> u32 {
        self.message_length
    }

    fn message_type(&self) -> u8 {
        self.message_type
    }
}

impl IntoBytes for Piece {
    fn into_bytes(self) -> Vec<u8> {
        let mut serialized_message: Vec<u8> = Vec::new();
        serialized_message.extend(self.message_length.to_be_bytes());
        serialized_message.push(self.message_type);
        serialized_message.extend(self.piece_index.to_be_bytes());
        serialized_message.extend(self.begin_offset_of_piece.to_be_bytes());
        serialized_message.extend(self.data);
        serialized_message
    }
}

impl FromBytes for Piece {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), Error> {
        if (bytes.len() as u32)
            < MessageType::Piece.base_length() + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES
        {
            return Err(Error::BytesArrayTooShortToContrainMessageFields);
        }

        let message_length = u32::from_be_bytes(
            bytes[0..4]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentMessageLength)?,
        );
        if message_length + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES != bytes.len() as u32
        {
            return Err(Error::MessageLengthDoesNotMatchWithExpectedOne);
        }

        let message_type = bytes[4];
        if message_type != MessageType::Piece.id() {
            return Err(Error::MessageTypeDoesNotMatchWithExpectedOne);
        }

        let piece_index = u32::from_be_bytes(
            bytes[5..9]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentPieceMessagePieceIndex)?,
        );

        let begin_offset_of_piece = u32::from_be_bytes(
            bytes[9..13]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentPieceMessageBeginOffset)?,
        );

        let data = bytes[13..bytes.len()].to_vec();

        Ok((
            Self {
                message_length,
                message_type,
                piece_index,
                begin_offset_of_piece,
                data,
            },
            (message_length + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES) as usize,
        ))
    }
}
