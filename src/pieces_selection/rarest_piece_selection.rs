use std::collections::{HashMap, VecDeque};

use bit_vec::BitVec;
use rand::seq::SliceRandom;

use crate::{http::Peer, pieces_selection::PriorityPiecesSelection};

use super::PieceSelection;

#[derive(Debug)]
pub struct RarestPiecesSelector;

impl PriorityPiecesSelection for RarestPiecesSelector {
    fn priority_pieces_selection(
        mybitfield: BitVec,
        peers_bitfields: HashMap<Peer, BitVec>,
    ) -> VecDeque<PieceSelection> {
        let mut occurrence_piece_selections: VecDeque<PieceSelection> = VecDeque::new();

        let mut piece_selections_indexed_by_piece_id: Vec<Vec<PieceSelection>> =
            vec![Vec::new(); mybitfield.len()];
        for (current_peer, current_bitvec) in peers_bitfields {
            for (piece_id, current_bit) in current_bitvec.iter().enumerate() {
                let piece_id: u32 = piece_id as u32;
                let current_piece_is_already_mine: bool = match mybitfield.get(piece_id as usize) {
                    Some(piece_is_present) => piece_is_present,
                    None => false,
                };
                if current_bit && !current_piece_is_already_mine {
                    // Update list of peers by piece_id
                    let mut current_peers =
                        piece_selections_indexed_by_piece_id[piece_id as usize].clone();
                    let current_piece_selection = PieceSelection::new(piece_id, current_peer);
                    current_peers.push(current_piece_selection);
                    piece_selections_indexed_by_piece_id[piece_id as usize] = current_peers;
                }
            }
        }

        // Sort the selections by occurrence
        piece_selections_indexed_by_piece_id.sort_by(|a, b| a.len().cmp(&b.len()));
        let piece_selections_ordered_by_frequency = piece_selections_indexed_by_piece_id;

        for piece_selections in piece_selections_ordered_by_frequency {
            // Choose randomly a PieceSelection to add to the returned queue
            let maybe_choosen_piece_selection = piece_selections.choose(&mut rand::thread_rng());
            match maybe_choosen_piece_selection {
                Some(choosen_piece_selection) => {
                    occurrence_piece_selections.push_back(choosen_piece_selection.clone());
                }
                None => (),
            };
        }

        occurrence_piece_selections
    }
}
