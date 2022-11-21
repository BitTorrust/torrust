use crate::pwp::{
    identity_first_message_type_of, Bitfield, FromBytes, Handshake, Have, Interested, MessageType,
    NotInterested, Piece, Request, Unchoke,
};

use {
    crate::{
        http::Peer,
        pwp::{IntoBytes, Message},
        Error,
    },
    std::{
        io::{self, prelude::*},
        net::TcpStream,
    },
};

/// This is a temporary struct that changes the method receive from
///
/// TCPSession::receive(&self, buffer: &mut [u8]) -> Result<usize, io::Error>
///
/// to
///
/// TCPSession::receive(&mut self) -> Result<Option<Message>, io::Error>
///
/// It will be used to develop the other parts of the code while
/// the changes to the real TCPSession structure are not finished.
pub struct TCPSessionMock {
    peer: Peer,
    stream: TcpStream,
}

impl TCPSessionMock {
    pub fn connect(peer: Peer) -> Result<Self, Error> {
        let stream =
            TcpStream::connect(peer.socket_address()).map_err(|_| Error::FailedToConnectToPeer)?;
        Ok(Self {
            peer,
            stream: stream,
        })
    }

    fn stream(&self) -> &TcpStream {
        &self.stream
    }

    /// Returns the number of bytes sent
    pub fn send(&self, bittorrent_message: impl IntoBytes) -> Result<usize, io::Error> {
        self.stream().write(&(bittorrent_message.into_bytes()))
    }

    fn parse_bitfield_message(&self, bytes: &mut Vec<u8>) -> Result<Option<Message>, Error> {
        let remaining_bytes_to_read_length = u32::from_be_bytes(
            bytes[0..4]
                .try_into()
                .map_err(|_| Error::FailedToParseReceivedBitfieldLength)?,
        ) - 1;
        let mut remaining_bytes_to_read = Vec::new();
        remaining_bytes_to_read.resize(remaining_bytes_to_read_length as usize, 0);

        self.stream()
            .read(&mut remaining_bytes_to_read)
            .map_err(|_| Error::FailedToReadFromSocket)?;
        bytes.extend_from_slice(&remaining_bytes_to_read);
        match Bitfield::from_bytes(&bytes) {
            Ok(bitfield_and_size) => Ok(Some(Message::Bitfield(bitfield_and_size.0))),
            Err(error) => Err(error),
        }
    }

    fn parse_have_message(&self, bytes: &mut Vec<u8>) -> Result<Option<Message>, Error> {
        let mut remaining_bytes_to_read: [u8; 4] = [0; 4];
        self.stream()
            .read(&mut remaining_bytes_to_read)
            .map_err(|_| Error::FailedToReadFromSocket)?;
        bytes.extend_from_slice(&remaining_bytes_to_read);
        match Have::from_bytes(&bytes) {
            Ok(have_and_size) => Ok(Some(Message::Have(have_and_size.0))),
            Err(error) => Err(error),
        }
    }

    fn parse_request_message(&self, bytes: &mut Vec<u8>) -> Result<Option<Message>, Error> {
        let mut remaining_bytes_to_read: [u8; 24] = [0; 3 * 8];
        self.stream()
            .read(&mut remaining_bytes_to_read)
            .map_err(|_| Error::FailedToReadFromSocket)?;
        bytes.extend_from_slice(&remaining_bytes_to_read);
        match Request::from_bytes(&bytes) {
            Ok(request_and_size) => Ok(Some(Message::Request(request_and_size.0))),
            Err(error) => Err(error),
        }
    }

    fn parse_piece_message(&self, bytes: &mut Vec<u8>) -> Result<Option<Message>, Error> {
        let remaining_bytes_to_read_length = u32::from_be_bytes(
            bytes[0..4]
                .try_into()
                .map_err(|_| Error::FailedToParseReceivedPieceLength)?,
        ) - 1;

        let mut remaining_bytes_to_read = Vec::new();
        remaining_bytes_to_read.resize(remaining_bytes_to_read_length as usize, 0);

        self.stream()
            .read(&mut remaining_bytes_to_read)
            .map_err(|_| Error::FailedToReadFromSocket)?;
        bytes.extend_from_slice(&remaining_bytes_to_read);
        match Piece::from_bytes(&bytes) {
            Ok(piece_and_size) => Ok(Some(Message::Piece(piece_and_size.0))),
            Err(error) => Err(error),
        }
    }

    fn parse_message(
        &self,
        message: MessageType,
        bytes: &mut Vec<u8>,
    ) -> Result<Option<Message>, Error> {
        match message {
            MessageType::Bitfield => self.parse_bitfield_message(bytes),
            MessageType::Unchoke => Ok(Some(Message::Unchoke(Unchoke::new()))),
            MessageType::Interested => Ok(Some(Message::Interested(Interested::new()))),
            MessageType::NotInterested => Ok(Some(Message::NotInterested(NotInterested::new()))),
            MessageType::Have => self.parse_have_message(bytes),
            MessageType::Request => self.parse_request_message(bytes),
            MessageType::Piece => self.parse_piece_message(bytes),
        }
    }

    /// Write the received bytes in the buffer
    /// Returns the number of bytes received
    pub fn receive(&mut self) -> Result<Option<Message>, Error> {
        // check if it is a handshake
        // PWP message are all starting with a 4 bytes representing the message length
        let mut zero_to_third_read_bytes: [u8; 4] = [0; 4];
        self.stream
            .read(&mut zero_to_third_read_bytes)
            .map_err(|_| Error::FailedToReadFromSocket)?;

        // Keep alive (PWP protocol) handling
        if zero_to_third_read_bytes == [0, 0, 0, 0] {
            // Keep alive message case
            return Ok(Some(Message::KeepAlive));
        }

        let mut fourth_read_byte: [u8; 1] = [0];
        self.stream
            .read(&mut fourth_read_byte)
            .map_err(|_| Error::FailedToReadFromSocket)?;
        let mut zero_to_fourth_read_bytes = [
            zero_to_third_read_bytes[0],
            zero_to_third_read_bytes[1],
            zero_to_third_read_bytes[2],
            zero_to_third_read_bytes[3],
            fourth_read_byte[0],
        ];

        // Handshake handling
        let mut handshake_protocol_name = Handshake::BITTORRENT_VERSION_1_PROTOCOL_NAME.chars();
        let expected_four_first_byte_of_handshake = [
            Handshake::BITTORRENT_VERSION_1_PROTOCOL_NAME_LENGTH,
            handshake_protocol_name.next().unwrap() as u8,
            handshake_protocol_name.next().unwrap() as u8,
            handshake_protocol_name.next().unwrap() as u8,
            handshake_protocol_name.next().unwrap() as u8,
        ];
        if zero_to_fourth_read_bytes == expected_four_first_byte_of_handshake {
            // Handshake message case
            let mut handshake_bytes: Vec<u8> = Vec::new();
            handshake_bytes.resize(
                Handshake::HANDSHAKE_VERSION_1_MESSAGE_LENGTH
                    - expected_four_first_byte_of_handshake.len(),
                0,
            );
            self.stream()
                .read(&mut handshake_bytes)
                .map_err(|_| Error::FailedToReadFromSocket)?;
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&mut zero_to_fourth_read_bytes);
            bytes.extend_from_slice(&mut handshake_bytes);

            // Create handshake message from bytes
            let handshake = match Handshake::from_bytes(&bytes) {
                Ok(handshake_and_size) => handshake_and_size.0,
                Err(error) => return Err(error),
            };
            return Ok(Some(Message::Handshake(handshake)));
        }

        // PWP messages
        // if not handshake, it is a pwp message
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&zero_to_fourth_read_bytes);
        match identity_first_message_type_of(&bytes) {
            Ok(message) => self.parse_message(message, &mut bytes),
            Err(error) => Err(error),
        }
    }
}
