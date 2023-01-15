use std::collections::HashMap;

use bit_vec::BitVec;

use crate::{
    http::Peer,
    pieces_selection::{PieceSelection, PiecesSelection},
};

#[derive(Debug)]
pub struct SimpleSelector;

impl PiecesSelection for SimpleSelector {
    fn pieces_selection(
        mybitfield: BitVec,
        peers_bitfields: HashMap<Peer, BitVec>,
    ) -> Vec<PieceSelection> {
        let mut piece_id_to_maybe_peer = Vec::new();

        for (current_peer, current_bitvec) in &peers_bitfields {
            for (piece_id, current_bit) in current_bitvec.iter().enumerate() {
                let piece_id = piece_id as u32;
                let current_piece_is_already_mine: bool = match mybitfield.get(piece_id as usize) {
                    Some(piece_is_present) => piece_is_present,
                    None => false,
                };

                if current_bit && !current_piece_is_already_mine {
                    let piece_selection = PieceSelection::new(piece_id, current_peer.to_owned());
                    piece_id_to_maybe_peer.push(piece_selection);
                }
            }
        }

        piece_id_to_maybe_peer
    }
}
