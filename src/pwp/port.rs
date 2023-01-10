use crate::pwp::{from_bytes, FromBytes, IntoBytes, MandatoryBitTorrentMessageFields, MessageType};
use crate::Error;

#[derive(Debug)]
pub struct Port {
    message_length: u32,
    message_type: u8,
    listen_port: u16,
}

impl Port {
    pub fn new(listen_port: u16) -> Self {
        Self {
            message_length: MessageType::Port.base_length(),
            message_type: MessageType::Port.id(),
            listen_port,
        }
    }

    pub fn message_length(&self) -> u32 {
        self.message_length
    }

    pub fn message_type(&self) -> u8 {
        self.message_type
    }

    pub fn listen_port(&self) -> u16 {
        self.listen_port
    }
}

impl MandatoryBitTorrentMessageFields for Port {
    fn message_length(&self) -> u32 {
        self.message_length
    }

    fn message_type(&self) -> u8 {
        self.message_type
    }
}

impl IntoBytes for Port {
    fn into_bytes(self) -> Vec<u8> {
        let mut serialized_message: Vec<u8> = Vec::new();
        serialized_message.extend(self.message_length.to_be_bytes());
        serialized_message.push(self.message_type);
        serialized_message.extend(self.listen_port.to_be_bytes());
        serialized_message
    }
}

impl FromBytes for Port {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), Error> {
        if (bytes.len() as u32)
            < (MessageType::Port.base_length() + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES)
        {
            return Err(Error::BytesArrayTooShort);
        }

        let message_length = u32::from_be_bytes(
            bytes[0..4]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentMessageLength)?,
        );
        if message_length != MessageType::Port.base_length() {
            return Err(Error::MessageLengthDoesNotMatchWithExpectedOne);
        }

        let message_type = bytes[4];
        if message_type != MessageType::Port.id() {
            return Err(Error::MessageTypeDoesNotMatchWithExpectedOne);
        }

        let listen_port = u16::from_be_bytes(
            bytes[5..7]
                .try_into()
                .map_err(|_| Error::FailedToParseBitTorrentPortMessagePieceIndex)?,
        );

        Ok((
            Self {
                message_length,
                message_type,
                listen_port,
            },
            (message_length + from_bytes::PWP_MESSAGE_LENGTH_FIELD_SIZE_IN_BYTES) as usize,
        ))
    }
}
