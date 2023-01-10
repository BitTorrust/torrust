use crate::{
    Bitfield, Cancel, Choke, Error, FromBytes, Handshake, Have, Interested, KeepAlive, Message,
    MessageType, NotInterested, Piece, Port, Request, Unchoke,
};

use super::TcpSession;

#[derive(Debug)]
pub struct MessageParser;

impl MessageParser {
    fn parse_message_length(
        tcp_session: &TcpSession,
        length_field_size: usize,
    ) -> Result<u32, Error> {
        let mut peeked_bytes = Vec::new();
        peeked_bytes.resize(length_field_size, 0);
        tcp_session
            .stream()
            .peek(&mut peeked_bytes)
            .map_err(|_| Error::FailedToReadFromSocket)?;
        let length = u32::from_be_bytes(
            peeked_bytes[0..length_field_size]
                .try_into()
                .map_err(|_| Error::FailedToParseLengthFieldSize)?,
        );
        Ok(length)
    }

    fn parse_bitfield_message(tcp_session: &TcpSession) -> Result<Option<Message>, Error> {
        // Get bytes size to read from buffer
        let variable_length = MessageParser::parse_message_length(
            tcp_session,
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE as usize,
        )?;
        let message_length = MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + variable_length;

        // Read the entire message from the buffer
        let bitfield_bytes = tcp_session.read_buffer(message_length as usize)?;

        // Create Bitfield message from bytes
        match Bitfield::from_bytes(&bitfield_bytes) {
            Ok(bitfield_and_size) => Ok(Some(Message::Bitfield(bitfield_and_size.0))),
            Err(error) => Err(error),
        }
    }

    fn parse_have_message(tcp_session: &TcpSession) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length =
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + MessageType::Have.base_length();

        // Read the entire message from the buffer
        let have_bytes = tcp_session.read_buffer(message_length as usize)?;

        // Create Have message from bytes
        match Have::from_bytes(&have_bytes) {
            Ok(have_and_size) => Ok(Some(Message::Have(have_and_size.0))),
            Err(error) => Err(error),
        }
    }

    fn parse_request_message(tcp_session: &TcpSession) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length =
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + MessageType::Request.base_length();

        // Read the entire message from the buffer
        let request_bytes = tcp_session.read_buffer(message_length as usize)?;

        // Create Request message from bytes
        match Request::from_bytes(&request_bytes) {
            Ok(request_and_size) => Ok(Some(Message::Request(request_and_size.0))),
            Err(error) => Err(error),
        }
    }

    fn parse_piece_message(tcp_session: &TcpSession) -> Result<Option<Message>, Error> {
        // Get bytes size to read from buffer
        let variable_length = MessageParser::parse_message_length(
            tcp_session,
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE as usize,
        )?;
        let message_length = MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + variable_length;

        // Read the entire message from the buffer
        let piece_bytes = tcp_session.read_buffer(message_length as usize)?;

        // Create Piece message from bytes
        match Piece::from_bytes(&piece_bytes) {
            Ok(piece_and_size) => Ok(Some(Message::Piece(piece_and_size.0))),
            Err(error) => Err(error),
        }
    }

    pub fn parse_handshake_message(tcp_session: &TcpSession) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length = Handshake::HANDSHAKE_VERSION_1_MESSAGE_LENGTH;

        // Read the entire message from the buffer
        let handshake_bytes = tcp_session.read_buffer(message_length as usize)?;

        // Create Handshake message from bytes
        let handshake = match Handshake::from_bytes(&handshake_bytes) {
            Ok(handshake_and_size) => handshake_and_size.0,
            Err(error) => return Err(error),
        };
        Ok(Some(Message::Handshake(handshake)))
    }

    fn parse_unchoke_message(tcp_session: &TcpSession) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length =
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + MessageType::Unchoke.base_length();

        // Read the entire message from the buffer
        let unchoke_bytes = tcp_session.read_buffer(message_length as usize)?;

        // Create Unchoke message from bytes
        match Unchoke::from_bytes(&unchoke_bytes) {
            Ok(unchoke_and_size) => Ok(Some(Message::Unchoke(unchoke_and_size.0))),
            Err(error) => return Err(error),
        }
    }

    fn parse_interested_message(tcp_session: &TcpSession) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length =
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + MessageType::Interested.base_length();

        // Read the entire message from the buffer
        let interested_bytes = tcp_session.read_buffer(message_length as usize)?;

        // Create Interested message from bytes
        match Interested::from_bytes(&interested_bytes) {
            Ok(interested_and_size) => Ok(Some(Message::Interested(interested_and_size.0))),
            Err(error) => return Err(error),
        }
    }

    fn parse_keep_alive_message(tcp_session: &TcpSession) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length =
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + MessageType::KeepAlive.base_length();

        // Read the entire message from the buffer
        let keep_alive_bytes = tcp_session.read_buffer(message_length as usize)?;

        // Create Keep-Alive message from bytes
        match KeepAlive::from_bytes(&keep_alive_bytes) {
            Ok(keep_alive_and_size) => Ok(Some(Message::KeepAlive(keep_alive_and_size.0))),
            Err(error) => return Err(error),
        }
    }

    fn parse_not_interested_message(tcp_session: &TcpSession) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length =
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + MessageType::NotInterested.base_length();

        // Read the entire message from the buffer
        let not_interested_bytes = tcp_session.read_buffer(message_length as usize)?;

        // Create Interested message from bytes
        match NotInterested::from_bytes(&not_interested_bytes) {
            Ok(not_interested_and_size) => {
                Ok(Some(Message::NotInterested(not_interested_and_size.0)))
            }
            Err(error) => return Err(error),
        }
    }

    fn parse_choke_message(tcp_session: &TcpSession) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length =
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + MessageType::Choke.base_length();

        // Read the entire message from the buffer
        let choke_bytes = tcp_session.read_buffer(message_length as usize)?;

        // Create Choke message from bytes
        match Choke::from_bytes(&choke_bytes) {
            Ok(choke_and_size) => Ok(Some(Message::Choke(choke_and_size.0))),
            Err(error) => return Err(error),
        }
    }

    fn parse_cancel_message(tcp_session: &TcpSession) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length =
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + MessageType::Request.base_length();

        // Read the entire message from the buffer
        let cancel_bytes = tcp_session.read_buffer(message_length as usize)?;

        // Create Cancel message from bytes
        match Cancel::from_bytes(&cancel_bytes) {
            Ok(cancel_and_size) => Ok(Some(Message::Cancel(cancel_and_size.0))),
            Err(error) => Err(error),
        }
    }

    fn parse_port_message(tcp_session: &TcpSession) -> Result<Option<Message>, Error> {
        // Get bytes size to read
        let message_length =
            MessageType::PWP_MESSAGE_LENGTH_FIELD_SIZE + MessageType::Port.base_length();

        // Read the entire message from the buffer
        let port_bytes = tcp_session.read_buffer(message_length as usize)?;

        // Create Port message from bytes
        match Port::from_bytes(&port_bytes) {
            Ok(port_and_size) => Ok(Some(Message::Port(port_and_size.0))),
            Err(error) => Err(error),
        }
    }

    pub fn parse_message(
        tcp_session: &TcpSession,
        message: MessageType,
    ) -> Result<Option<Message>, Error> {
        match message {
            MessageType::Bitfield => MessageParser::parse_bitfield_message(tcp_session),
            MessageType::Choke => MessageParser::parse_choke_message(tcp_session),
            MessageType::Unchoke => MessageParser::parse_unchoke_message(tcp_session),
            MessageType::Interested => MessageParser::parse_interested_message(tcp_session),
            MessageType::NotInterested => MessageParser::parse_not_interested_message(tcp_session),
            MessageType::Have => MessageParser::parse_have_message(tcp_session),
            MessageType::Request => MessageParser::parse_request_message(tcp_session),
            MessageType::Piece => MessageParser::parse_piece_message(tcp_session),
            MessageType::KeepAlive => MessageParser::parse_keep_alive_message(tcp_session), // never called
            MessageType::Cancel => MessageParser::parse_cancel_message(tcp_session),
            MessageType::Port => MessageParser::parse_port_message(tcp_session),
        }
    }
}
