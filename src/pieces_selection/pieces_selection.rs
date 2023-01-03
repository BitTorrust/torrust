use bit_vec::BitVec;
use std::collections::HashMap;

use crate::http::Peer;

pub trait PiecesSelection {
    /// Returns a associative table matching a piece_id with a peer for requesting the piece associated with the piece_id
    fn pieces_selection(
        mybitfield: BitVec,
        peers_bitfields: HashMap<Peer, BitVec>,
    ) -> HashMap<u32, Option<Peer>>;
}
