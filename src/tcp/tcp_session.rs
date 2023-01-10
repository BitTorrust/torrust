use crate::{
    pwp::{
        identity_first_message_type_of, Bitfield, FromBytes, Handshake, Have, Interested,
        MessageType, NotInterested, Piece, Request, Unchoke,
    },
    Choke, KeepAlive,
};

use super::MessageParser;

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

    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }

    /// Returns the number of bytes sent
    pub fn send(&self, bittorrent_message: impl IntoBytes) -> Result<usize, io::Error> {
        self.stream().write(&(bittorrent_message.into_bytes()))
    }

    pub fn read_buffer(&self, size: usize) -> Result<Vec<u8>, Error> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.resize(size as usize, 0);
        self.stream()
            .read(&mut bytes)
            .map_err(|_| Error::FailedToReadFromSocket)?;
        Ok(bytes)
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
            return Ok(Some(Message::KeepAlive(KeepAlive::new())));
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
            return MessageParser::parse_handshake_message(&self);
        }

        // PWP messages
        // if not handshake, it is probably a Peer Wire Protocol message
        match identity_first_message_type_of(&zero_to_fourth_read_bytes) {
            Ok(message) => MessageParser::parse_message(&self, message),
            Err(error) => {
                log::error!("Unexpected TCP data: {:?}", zero_to_fourth_read_bytes);

                Err(error)
            }
        }
    }
}
