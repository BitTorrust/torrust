use std::collections::HashMap;

use bit_vec::BitVec;

use crate::{http::Peer, pieces_selection::PiecesSelection};

#[derive(Debug)]
pub struct SimpleSelector;

impl PiecesSelection for SimpleSelector {
    fn pieces_selection(
        mybitfield: BitVec,
        peers_bitfields: HashMap<Peer, BitVec>,
    ) -> HashMap<u32, Option<Peer>> {
        let mut piece_id_to_maybe_peer: HashMap<u32, Option<Peer>> = HashMap::new();

        for (current_peer, current_bitvec) in &peers_bitfields {
            for (piece_id, current_bit) in current_bitvec.iter().enumerate() {
                let piece_id: u32 = piece_id as u32;
                let current_piece_is_already_mine: bool = match mybitfield.get(piece_id as usize) {
                    Some(piece_is_present) => piece_is_present,
                    None => false,
                };
                match piece_id_to_maybe_peer.get(&piece_id) {
                    None => {
                        // Initiate the peer values
                        if current_bit && !current_piece_is_already_mine {
                            piece_id_to_maybe_peer.insert(piece_id, Some(current_peer.clone()));
                        } else {
                            piece_id_to_maybe_peer.insert(piece_id, None);
                        }
                    }
                    Some(None) => {
                        // Update the peer value if it is a None
                        if !current_piece_is_already_mine {
                            piece_id_to_maybe_peer.insert(piece_id, Some(current_peer.clone()));
                        }
                    }
                    Some(Some(_)) => (),
                }
            }
        }

        piece_id_to_maybe_peer
    }
}
