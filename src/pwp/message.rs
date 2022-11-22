use crate::pwp;

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
