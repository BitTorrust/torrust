use std::cmp::Ordering;

use crate::http::Peer;

#[derive(Debug, Clone)]
pub struct PieceSelection {
    piece_id: u32,
    peer: Peer,
}

impl PieceSelection {
    pub fn piece_id(&self) -> u32 {
        self.piece_id
    }

    pub fn peer(&self) -> Peer {
        self.peer
    }

    pub fn new(piece_id: u32, peer: Peer) -> Self {
        PieceSelection { piece_id, peer }
    }
}

impl Ord for PieceSelection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.piece_id().cmp(&other.piece_id())
    }
}

impl PartialOrd for PieceSelection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PieceSelection {
    fn eq(&self, other: &Self) -> bool {
        self.piece_id() == other.piece_id()
    }
}

impl Eq for PieceSelection {}
