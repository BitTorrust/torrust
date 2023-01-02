#[cfg(test)]
pub mod unittest {
    use std::{
        collections::HashMap,
        net::{IpAddr, Ipv4Addr, SocketAddr},
    };

    use bit_vec::BitVec;

    use crate::{
        http::Peer,
        pieces_selection::{rarest_piece_selection::RarestPiecesSelector, PiecesSelection},
    };

    #[test]
    pub fn select_pieces_with_one_peer_having_all_pieces() {
        let mut peers_bitfields: HashMap<Peer, BitVec> = HashMap::new();
        let seeder: Peer = Peer::from_socket_address(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            6999,
        ));
        let bitfield_length = 9;
        let seeder_bitfield = BitVec::from_elem(bitfield_length, true);
        peers_bitfields.insert(seeder, seeder_bitfield);

        let mybitfield: BitVec = BitVec::from_elem(bitfield_length, false);
        let selection: HashMap<u32, Option<Peer>> =
            RarestPiecesSelector::pieces_selection(mybitfield, peers_bitfields);

        let mut expected_selection: HashMap<u32, Option<Peer>> = HashMap::new();
        for piece_id in 0..bitfield_length {
            expected_selection.insert(piece_id as u32, Some(seeder));
        }

        assert_eq!(selection, expected_selection);
    }

    #[test]
    pub fn select_pieces_with_two_peers_having_half_pieces_each() {
        let mut peers_bitfields: HashMap<Peer, BitVec> = HashMap::new();
        let bitfield_length = 9;
        let first_part_seeder: Peer = Peer::from_socket_address(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            6999,
        ));
        let mut first_part_seeder_bitfield = BitVec::from_elem(bitfield_length, true);
        first_part_seeder_bitfield.set(5, false);
        first_part_seeder_bitfield.set(6, false);
        first_part_seeder_bitfield.set(7, false);
        first_part_seeder_bitfield.set(8, false);

        let second_part_seeder: Peer = Peer::from_socket_address(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            6998,
        ));
        let mut second_part_seeder_bitfield = BitVec::from_elem(bitfield_length, true);
        second_part_seeder_bitfield.set(0, false);
        second_part_seeder_bitfield.set(1, false);
        second_part_seeder_bitfield.set(2, false);
        second_part_seeder_bitfield.set(3, false);
        second_part_seeder_bitfield.set(4, false);

        peers_bitfields.insert(first_part_seeder, first_part_seeder_bitfield);
        peers_bitfields.insert(second_part_seeder, second_part_seeder_bitfield);

        let mybitfield: BitVec = BitVec::from_elem(bitfield_length, false);
        let selection: HashMap<u32, Option<Peer>> =
            RarestPiecesSelector::pieces_selection(mybitfield, peers_bitfields);

        let mut expected_selection: HashMap<u32, Option<Peer>> = HashMap::new();
        for piece_id in 0..5 {
            expected_selection.insert(piece_id as u32, Some(first_part_seeder));
        }
        for piece_id in 5..bitfield_length {
            expected_selection.insert(piece_id as u32, Some(second_part_seeder));
        }

        assert_eq!(selection, expected_selection);
    }

    #[test]
    pub fn select_pieces_with_two_peers_having_overlapping_pieces() {
        let mut peers_bitfields: HashMap<Peer, BitVec> = HashMap::new();
        let bitfield_length = 4;
        let first_part_seeder: Peer = Peer::from_socket_address(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            0001,
        ));
        let mut first_part_seeder_bitfield = BitVec::from_elem(bitfield_length, true);
        first_part_seeder_bitfield.set(3, false);

        let second_part_seeder: Peer = Peer::from_socket_address(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            0002,
        ));
        let mut second_part_seeder_bitfield = BitVec::from_elem(bitfield_length, true);
        second_part_seeder_bitfield.set(0, false);
        second_part_seeder_bitfield.set(1, false);

        peers_bitfields.insert(first_part_seeder, first_part_seeder_bitfield);
        peers_bitfields.insert(second_part_seeder, second_part_seeder_bitfield);

        let mybitfield: BitVec = BitVec::from_elem(bitfield_length, false);
        let selection: HashMap<u32, Option<Peer>> =
            RarestPiecesSelector::pieces_selection(mybitfield, peers_bitfields);

        let mut expected_selection: HashMap<u32, Option<Peer>> = HashMap::new();
        for piece_id in 0..5 {
            expected_selection.insert(piece_id as u32, Some(first_part_seeder));
        }
        for piece_id in 5..bitfield_length {
            expected_selection.insert(piece_id as u32, Some(second_part_seeder));
        }

        for (piece_id, maybe_peer) in &selection {
            match piece_id {
                0..=1 => {
                    assert_eq!(maybe_peer.unwrap(), first_part_seeder);
                }
                2 => match maybe_peer {
                    Some(_) => assert!(true),
                    None => assert!(false),
                },
                3 => {
                    assert_eq!(maybe_peer.unwrap(), second_part_seeder);
                }
                _ => (),
            }
        }
    }

    #[test]
    pub fn select_pieces_with_one_peer_having_one_missing_that_we_have() {
        let mut peers_bitfields: HashMap<Peer, BitVec> = HashMap::new();
        let bitfield_length = 4;

        let seeder: Peer = Peer::from_socket_address(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            0002,
        ));
        let mut seeder_bitfield = BitVec::from_elem(bitfield_length, true);
        seeder_bitfield.set(0, false);

        peers_bitfields.insert(seeder, seeder_bitfield);

        let mut mybitfield: BitVec = BitVec::from_elem(bitfield_length, false);
        mybitfield.set(0, true);
        let selection: HashMap<u32, Option<Peer>> =
            RarestPiecesSelector::pieces_selection(mybitfield, peers_bitfields);

        let mut expected_selection: HashMap<u32, Option<Peer>> = HashMap::new();
        for piece_id in 0..5 {
            expected_selection.insert(piece_id as u32, Some(seeder));
        }

        for (piece_id, maybe_peer) in &selection {
            match piece_id {
                0 => match maybe_peer {
                    Some(_) => assert!(false),
                    None => assert!(true), // we already have the 0-indexed piece
                },
                1..=3 => {
                    assert_eq!(maybe_peer.unwrap(), seeder);
                }
                _ => (),
            }
        }
    }

    #[test]
    pub fn select_pieces_with_one_missing_piece_globally() {
        let mut peers_bitfields: HashMap<Peer, BitVec> = HashMap::new();
        let bitfield_length = 4;

        let seeder: Peer = Peer::from_socket_address(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            0002,
        ));
        let mut seeder_bitfield = BitVec::from_elem(bitfield_length, true);
        seeder_bitfield.set(0, false);

        peers_bitfields.insert(seeder, seeder_bitfield);

        let mybitfield: BitVec = BitVec::from_elem(bitfield_length, false);
        let selection: HashMap<u32, Option<Peer>> =
            RarestPiecesSelector::pieces_selection(mybitfield, peers_bitfields);

        let mut expected_selection: HashMap<u32, Option<Peer>> = HashMap::new();
        for piece_id in 0..5 {
            expected_selection.insert(piece_id as u32, Some(seeder));
        }

        for (piece_id, maybe_peer) in &selection {
            match piece_id {
                0 => match maybe_peer {
                    Some(_) => assert!(false),
                    None => assert!(true), // the 0-indexed piece is missing for all peers
                },
                1..=3 => {
                    assert_eq!(maybe_peer.unwrap(), seeder);
                }
                _ => (),
            }
        }
    }
}
