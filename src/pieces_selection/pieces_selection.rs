use bit_vec::BitVec;
use std::collections::{HashMap, VecDeque};

use super::PieceSelection;
use crate::http::Peer;

pub trait PiecesSelection {
    /// Returns a associative table matching a piece_id with a peer for requesting the piece associated with the piece_id
    fn pieces_selection(
        mybitfield: BitVec,
        peers_bitfields: HashMap<Peer, BitVec>,
    ) -> Vec<PieceSelection>;
}

// TODO: delete this and use PiecesSelection instead.
pub trait PriorityPiecesSelection {
    fn priority_pieces_selection(
        mybitfield: BitVec,
        peers_bitfields: HashMap<Peer, BitVec>,
    ) -> VecDeque<PieceSelection>;
}
