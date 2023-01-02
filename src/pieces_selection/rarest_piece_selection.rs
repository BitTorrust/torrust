use std::collections::{BTreeMap, HashMap};

use bit_vec::BitVec;
use rand::seq::SliceRandom;

use crate::{http::Peer, pieces_selection::PiecesSelection};

#[derive(Debug)]
pub struct RarestPiecesSelector;

impl PiecesSelection for RarestPiecesSelector {
    fn pieces_selection(
        mybitfield: BitVec,
        peers_bitfields: HashMap<Peer, BitVec>,
    ) -> HashMap<u32, Option<Peer>> {
        let mut piece_id_to_maybe_peer: HashMap<u32, Option<Peer>> = HashMap::new();

        let mut piece_to_peers: HashMap<u32, Vec<Peer>> = HashMap::new();

        // Make statistics to sort the frequencies of pieces
        let mut pieces_occurrences: Vec<u32> = Vec::new();
        for (current_peer, current_bitvec) in peers_bitfields {
            // Resize pieces_occurrences
            if pieces_occurrences.len() < current_bitvec.len() {
                pieces_occurrences.resize(current_bitvec.len(), 0);
            }
            for (piece_id, current_bit) in current_bitvec.iter().enumerate() {
                let piece_id: u32 = piece_id as u32;
                let current_piece_is_already_mine: bool = match mybitfield.get(piece_id as usize) {
                    Some(piece_is_present) => piece_is_present,
                    None => false,
                };
                if current_bit && !current_piece_is_already_mine {
                    pieces_occurrences[piece_id as usize] += 1;
                    match piece_to_peers.get(&piece_id) {
                        Some(peers) => {
                            let mut new_peers = peers.clone();
                            new_peers.push(current_peer);
                            piece_to_peers.insert(piece_id, new_peers);
                        }
                        None => {
                            let mut new_peers: Vec<Peer> = Vec::new();
                            new_peers.push(current_peer);
                            piece_to_peers.insert(piece_id, new_peers);
                        }
                    }
                }
            }
        }

        println!("pieces_occurrences {:?}", pieces_occurrences);
        println!("piece_to_peers {:?}", piece_to_peers);

        // Sort the occurrences
        let mut occurrence_to_piece_ids: HashMap<u32, Vec<u32>> = HashMap::new();
        for (piece_id, occurrence) in pieces_occurrences.iter().enumerate() {
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
                        let maybe_choosen_peer = match piece_to_peers.get(piece_id) {
                            Some(peers) => peers.choose(&mut rand::thread_rng()),
                            None => None,
                        };
                        let peer = Some(maybe_choosen_peer.unwrap().clone());
                        piece_id_to_maybe_peer.insert(*piece_id, peer);
                    }
                }
                None => (),
            };
        }
        println!("piece_id_to_maybe_peer {:?}", piece_id_to_maybe_peer);

        piece_id_to_maybe_peer
    }
}
