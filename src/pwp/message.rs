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
    KeepAlive,
}

impl Message {
    pub fn into_inner(self) -> Box<dyn IntoBytes + 'static> {
        match self {
            Message::Handshake(m) => Box::new(m),
            Message::Bitfield(m) => Box::new(m),
            Message::Have(m) => Box::new(m),
            Message::Interested(m) => Box::new(m),
            Message::NotInterested(m) => Box::new(m),
            Message::Piece(m) => Box::new(m),
            Message::Request(m) => Box::new(m),
            Message::Unchoke(m) => Box::new(m),
        }
    }
}
