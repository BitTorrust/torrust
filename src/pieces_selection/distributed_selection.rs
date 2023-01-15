use std::collections::HashMap;

use bit_vec::BitVec;

use {
    crate::{
        http::Peer,
        pieces_selection::{PieceSelection, PiecesSelection},
    },
    rand::{seq::SliceRandom, thread_rng},
};

#[derive(Debug)]
pub struct DistributedSelector;

impl DistributedSelector {
    fn bitfield_has_piece(bitfield: &BitVec, piece: usize) -> bool {
        if let Some(true) = bitfield.get(piece) {
            true
        } else {
            false
        }
    }
}

impl PiecesSelection for DistributedSelector {
    fn pieces_selection(
        mybitfield: BitVec,
        peers_bitfields: HashMap<Peer, BitVec>,
    ) -> Vec<PieceSelection> {
        let mut target_peers = Vec::new();
        let mut index_to_use = 0;

        mybitfield
            .iter()
            .enumerate()
            .filter(|(_, b)| *b == false)
            .for_each(|(piece, _)| {
                let peers_having_this_piece: Vec<Peer> = peers_bitfields
                    .iter()
                    .filter(|(_, bitfield)| Self::bitfield_has_piece(*bitfield, piece))
                    .map(|(peer, _)| peer.to_owned())
                    .collect();

                if peers_having_this_piece.len() != 0 {
                    index_to_use = (index_to_use + 1) % peers_having_this_piece.len();
                    let peer_to_use = peers_having_this_piece[index_to_use].clone();

                    target_peers.push(PieceSelection::new(piece as u32, peer_to_use));
                }
            });

        target_peers.shuffle(&mut thread_rng());
        target_peers
    }
}
