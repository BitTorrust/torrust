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
        time::Duration,
    },
};

#[derive(Debug)]
pub struct TcpSession {
    stream: TcpStream,
}

impl TcpSession {
    pub fn from_stream(mut stream: TcpStream) -> Result<Self, Error> {
        Self::set_stream_parameters(&mut stream)?;

        Ok(Self { stream })
    }

    pub fn connect(peer: Peer) -> Result<Self, Error> {
        let address = peer.socket_address();
        let mut stream = TcpStream::connect(address).map_err(|_| Error::FailedToConnectToPeer)?;
        Self::set_stream_parameters(&mut stream)?;

        Ok(Self { stream })
    }

    fn set_stream_parameters(stream: &mut TcpStream) -> Result<(), Error> {
        stream
            .set_nonblocking(true)
            .map_err(|_| Error::FailedToSetSocketAsNonBlocking)?;

        stream
            .set_write_timeout(Some(Duration::from_millis(100)))
            .map_err(|_| Error::FailedToSetSocketWriteTimeout)?;
        // Disable Nagle Algorithm (useful for debugging now)
        stream
            .set_nodelay(true)
            .map_err(|_| Error::FailedToSetSocketAsNonBlocking)?;

        Ok(())
    }

    fn stream(&self) -> &TcpStream {
        &self.stream
    }

    /// Returns the number of bytes sent
    pub fn send(&self, bittorrent_message: impl IntoBytes) -> Result<usize, io::Error> {
        self.stream().write(&(bittorrent_message.into_bytes()))
    }

    fn parse_message_length(&self, length_field_size: usize) -> Result<u32, Error> {
        let mut length_bytes = Vec::new();
        length_bytes.resize(length_field_size, 0);
        self.stream()
            .peek(&mut length_bytes)
            .map_err(|_| Error::FailedToReadFromSocket)?;
        let length = u32::from_be_bytes(
            length_bytes[0..length_field_size]
                .try_into()
                .map_err(|_| Error::FailedToParseLengthFieldSize)?,
        );
        Ok(length)
    }

    fn read_buffer(&self, size: usize) -> Result<Vec<u8>, Error> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.resize(size as usize, 0);
        self.stream()
            .read(&mut bytes)
            .map_err(|_| Error::FailedToReadFromSocket)?;
        Ok(bytes)
    }

    fn parse_bitfield_message(&self) -> Result<Option<Message>, Error> {
        // Get bytes size to read from buffer
        let variable_length =
            self.parse_message_length(MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE as usize)? - 1;
        let message_length = MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE
            + MessageType::Bitfield.base_length()
            + variable_length;

        // Read the entire message from the buffer
        let bitfield_bytes = self.read_buffer(message_length as usize)?;

        // Create Bitfield message from bytes
        match Bitfield::from_bytes(&bitfield_bytes) {
            Ok(bitfield_and_size) => Ok(Some(Message::Bitfield(bitfield_and_size.0))),
            Err(error) => Err(error),
        }
    }

    fn parse_have_message(&self) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length = 4 + MessageType::Have.base_length();

        // Read the entire message from the buffer
        let have_bytes = self.read_buffer(message_length as usize)?;

        // Create Have message from bytes
        match Have::from_bytes(&have_bytes) {
            Ok(have_and_size) => Ok(Some(Message::Have(have_and_size.0))),
            Err(error) => Err(error),
        }
    }

    fn parse_request_message(&self) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length =
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + MessageType::Request.base_length();

        // Read the entire message from the buffer
        let request_bytes = self.read_buffer(message_length as usize)?;

        // Create Request message from bytes
        match Request::from_bytes(&request_bytes) {
            Ok(request_and_size) => Ok(Some(Message::Request(request_and_size.0))),
            Err(error) => Err(error),
        }
    }

    fn parse_piece_message(&self) -> Result<Option<Message>, Error> {
        // Get bytes size to read from buffer
        let variable_length =
            self.parse_message_length(MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE as usize)?;
        let message_length = MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + variable_length;

        // Read the entire message from the buffer
        let piece_bytes = self.read_buffer(message_length as usize)?;

        // Create Piece message from bytes
        match Piece::from_bytes(&piece_bytes) {
            Ok(piece_and_size) => Ok(Some(Message::Piece(piece_and_size.0))),
            Err(error) => Err(error),
        }
    }

    fn parse_handshake_message(&self) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length = Handshake::HANDSHAKE_VERSION_1_MESSAGE_LENGTH;

        // Read the entire message from the buffer
        let handshake_bytes = self.read_buffer(message_length as usize)?;

        // Create Handshake message from bytes
        let handshake = match Handshake::from_bytes(&handshake_bytes) {
            Ok(handshake_and_size) => handshake_and_size.0,
            Err(error) => return Err(error),
        };
        Ok(Some(Message::Handshake(handshake)))
    }

    fn parse_unchoke_message(&self) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length =
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + MessageType::Unchoke.base_length();

        // Read the entire message from the buffer
        let unchoke_bytes = self.read_buffer(message_length as usize)?;

        // Create Unchoke message from bytes
        match Unchoke::from_bytes(&unchoke_bytes) {
            Ok(unchoke_and_size) => Ok(Some(Message::Unchoke(unchoke_and_size.0))),
            Err(error) => return Err(error),
        }
    }

    fn parse_interested_message(&self) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length =
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + MessageType::Interested.base_length();

        // Read the entire message from the buffer
        let interested_bytes = self.read_buffer(message_length as usize)?;

        // Create Interested message from bytes
        match Interested::from_bytes(&interested_bytes) {
            Ok(interested_and_size) => Ok(Some(Message::Interested(interested_and_size.0))),
            Err(error) => return Err(error),
        }
    }

    fn parse_not_interested_message(&self) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length =
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + MessageType::NotInterested.base_length();

        // Read the entire message from the buffer
        let not_interested_bytes = self.read_buffer(message_length as usize)?;

        // Create Interested message from bytes
        match NotInterested::from_bytes(&not_interested_bytes) {
            Ok(not_interested_and_size) => {
                Ok(Some(Message::NotInterested(not_interested_and_size.0)))
            }
            Err(error) => return Err(error),
        }
    }

    fn parse_message(&self, message: MessageType) -> Result<Option<Message>, Error> {
        match message {
            MessageType::Bitfield => self.parse_bitfield_message(),
            MessageType::Unchoke => self.parse_unchoke_message(),
            MessageType::Interested => self.parse_interested_message(),
            MessageType::NotInterested => self.parse_not_interested_message(),
            MessageType::Have => self.parse_have_message(),
            MessageType::Request => self.parse_request_message(),
            MessageType::Piece => self.parse_piece_message(),
        }
    }

    /// Write the received bytes in the buffer
    /// Returns the received BitTorrent message or None (if there is no data in the buffer)
    pub fn receive(&mut self) -> Result<Option<Message>, Error> {
        // check if it is a handshake
        // PWP message are all starting with a 4 bytes representing the message length
        let mut zero_to_third_read_bytes: [u8; 4] = [0; 4];
        let number_of_bytes_read = match self.stream.peek(&mut zero_to_third_read_bytes) {
            Ok(read_bytes) => read_bytes,
            Err(_) => return Ok(None),
        };
        if number_of_bytes_read == 0 {
            return Ok(None);
        }

        // Keep alive (PWP protocol) handling
        if zero_to_third_read_bytes == [0, 0, 0, 0] {
            // Keep alive message case
            return Ok(Some(Message::KeepAlive));
        }

        // Handshake handling
        let mut zero_to_fourth_read_bytes: [u8; 5] = [0; 5];
        let number_of_bytes_read = match self.stream.peek(&mut zero_to_fourth_read_bytes) {
            Ok(read_bytes) => read_bytes,
            Err(_) => return Ok(None),
        };

        if number_of_bytes_read == 0 {
            return Ok(None);
        }

        let mut handshake_protocol_name = Handshake::BITTORRENT_VERSION_1_PROTOCOL_NAME.chars();
        let expected_four_first_byte_of_handshake = [
            Handshake::BITTORRENT_VERSION_1_PROTOCOL_NAME_LENGTH,
            handshake_protocol_name.next().unwrap() as u8,
            handshake_protocol_name.next().unwrap() as u8,
            handshake_protocol_name.next().unwrap() as u8,
            handshake_protocol_name.next().unwrap() as u8,
        ];
        if zero_to_fourth_read_bytes == expected_four_first_byte_of_handshake {
            return self.parse_handshake_message();
        }

        // PWP messages
        // if not handshake, it is a pwp message
        match identity_first_message_type_of(&zero_to_fourth_read_bytes) {
            Ok(message) => self.parse_message(message),
            Err(error) => {
                log::error!("Unexpected TCP data: {:?}", zero_to_fourth_read_bytes);

                Err(error)
            }
        }
    }
}
