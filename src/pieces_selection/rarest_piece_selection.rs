use std::collections::{BTreeMap, BinaryHeap, HashMap, VecDeque};

use bit_vec::BitVec;
use rand::seq::SliceRandom;

use crate::{
    http::Peer,
    pieces_selection::{PiecesSelection, PriorityPiecesSelection},
};

use super::{piece_selection, PieceSelection};

#[derive(Debug)]
pub struct RarestPiecesSelector;

impl PiecesSelection for RarestPiecesSelector {
    fn pieces_selection(
        mybitfield: BitVec,
        peers_bitfields: HashMap<Peer, BitVec>,
    ) -> HashMap<u32, Option<Peer>> {
        let mut ordered_by_occurrence_piece_selections: HashMap<u32, Option<Peer>> = HashMap::new();

        let mut piece_id_to_piece_selections: HashMap<u32, Vec<Peer>> = HashMap::new();

        // Make statistics to sort the frequencies of pieces
        let mut piece_id_to_its_occurrence: Vec<u32> = Vec::new();
        for (current_peer, current_bitvec) in peers_bitfields {
            // Resize piece_id_to_its_occurrence
            if piece_id_to_its_occurrence.len() < current_bitvec.len() {
                piece_id_to_its_occurrence.resize(current_bitvec.len(), 0);
            }
            for (piece_id, current_bit) in current_bitvec.iter().enumerate() {
                let piece_id: u32 = piece_id as u32;
                let current_piece_is_already_mine: bool = match mybitfield.get(piece_id as usize) {
                    Some(piece_is_present) => piece_is_present,
                    None => false,
                };
                if current_bit && !current_piece_is_already_mine {
                    piece_id_to_its_occurrence[piece_id as usize] += 1;
                    match piece_id_to_piece_selections.get(&piece_id) {
                        Some(peers) => {
                            let mut new_peers = peers.clone();
                            new_peers.push(current_peer);
                            piece_id_to_piece_selections.insert(piece_id, new_peers);
                        }
                        None => {
                            let mut new_peers: Vec<Peer> = Vec::new();
                            new_peers.push(current_peer);
                            piece_id_to_piece_selections.insert(piece_id, new_peers);
                        }
                    }
                }
            }
        }

        println!(
            "piece_id_to_its_occurrence {:?}",
            piece_id_to_its_occurrence
        );
        println!(
            "piece_id_to_piece_selections {:?}",
            piece_id_to_piece_selections
        );

        // Sort the occurrences
        let mut occurrence_to_piece_ids: HashMap<u32, Vec<u32>> = HashMap::new();
        for (piece_id, occurrence) in piece_id_to_its_occurrence.iter().enumerate() {
            let piece_id = piece_id as u32;
            let occurrence = occurrence.clone();
            match occurrence_to_piece_ids.get(&occurrence) {
                Some(piece_ids) => {
                    let mut new_piece_ids = piece_ids.clone();
                    new_piece_ids.push(piece_id);
                    occurrence_to_piece_ids.insert(occurrence, new_piece_ids);
                }
                None => {
                    let mut new_piece_ids: Vec<u32> = Vec::new();
                    new_piece_ids.push(piece_id);
                    occurrence_to_piece_ids.insert(occurrence, new_piece_ids);
                }
            }
        }
        println!("occurrence_to_piece_ids {:?}", occurrence_to_piece_ids);

        // Choose the rarest pieces "first"
        let ordered_occurrences: Vec<u32> = occurrence_to_piece_ids.clone().into_keys().collect();
        for occurrence in ordered_occurrences {
            // Handle the higher priority piece_ids
            if occurrence == 0 {
                continue;
            }
            match occurrence_to_piece_ids.get(&occurrence) {
                Some(piece_ids) => {
                    for piece_id in piece_ids {
                        // Choose randomly a Peer in the piece_ids available
                        let maybe_choosen_piece_selection =
                            match piece_id_to_piece_selections.get(piece_id) {
                                Some(peers) => peers.choose(&mut rand::thread_rng()),
                                None => None,
                            };
                        let peer = Some(maybe_choosen_piece_selection.unwrap().clone());
                        ordered_by_occurrence_piece_selections.insert(*piece_id, peer);
                    }
                }
                None => (),
            };
        }
        println!(
            "ordered_by_occurrence_piece_selections {:?}",
            ordered_by_occurrence_piece_selections
        );

        ordered_by_occurrence_piece_selections
    }
}

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
