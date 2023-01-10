use crate::pwp::{self, IntoBytes};

#[derive(Debug)]
pub enum Message {
    Handshake(pwp::Handshake),
    Bitfield(pwp::Bitfield),
    Have(pwp::Have),
    Interested(pwp::Interested),
    NotInterested(pwp::NotInterested),
    Piece(pwp::Piece),
    Request(pwp::Request),
    Unchoke(pwp::Unchoke),
    KeepAlive(pwp::KeepAlive),
}

impl IntoBytes for Message {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            Message::Handshake(m) => m.into_bytes(),
            Message::Bitfield(m) => m.into_bytes(),
            Message::Have(m) => m.into_bytes(),
            Message::Interested(m) => m.into_bytes(),
            Message::NotInterested(m) => m.into_bytes(),
            Message::Piece(m) => m.into_bytes(),
            Message::Request(m) => m.into_bytes(),
            Message::Unchoke(m) => m.into_bytes(),
            Message::KeepAlive(m) => m.into_bytes(),
        }
    }
}
