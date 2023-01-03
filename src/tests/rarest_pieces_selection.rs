#[cfg(test)]
pub mod unittest {
    use std::{
        collections::{HashMap, VecDeque},
        net::{IpAddr, Ipv4Addr, SocketAddr},
    };

    use bit_vec::BitVec;

    use crate::{
        http::Peer,
        pieces_selection::{
            rarest_piece_selection::RarestPiecesSelector, PieceSelection, PriorityPiecesSelection,
        },
    };

    #[test]
    pub fn select_rarest_pieces_with_one_peer_having_all_pieces() {
        let mut peers_bitfields: HashMap<Peer, BitVec> = HashMap::new();
        let seeder: Peer = Peer::from_socket_address(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            6999,
        ));
        let bitfield_length = 9;
        let seeder_bitfield = BitVec::from_elem(bitfield_length, true);
        peers_bitfields.insert(seeder, seeder_bitfield);

        let mybitfield: BitVec = BitVec::from_elem(bitfield_length, false);
        let selections: VecDeque<PieceSelection> =
            RarestPiecesSelector::priority_pieces_selection(mybitfield, peers_bitfields);

        for piece_id in 0..bitfield_length {
            let expected_current_piece_selection = PieceSelection::new(piece_id as u32, seeder);
            assert_eq!(selections[piece_id], expected_current_piece_selection);
        }
    }

    #[test]
    pub fn select_rarest_pieces_with_one_peer_having_one_missing_that_we_have() {
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
        let selections: VecDeque<PieceSelection> =
            RarestPiecesSelector::priority_pieces_selection(mybitfield, peers_bitfields);

        let mut expected_selections: VecDeque<PieceSelection> = VecDeque::new();
        for piece_id in 0..5 {
            let piece_selection = PieceSelection::new(piece_id as u32, seeder);
            expected_selections.push_back(piece_selection);
        }

        for selection in &selections {
            match selection.piece_id() {
                0 => assert!(true),
                1..=3 => assert_eq!(selection.peer(), seeder),
                _ => assert!(false),
            }
        }
    }

    #[test]
    pub fn select_rarest_pieces_with_one_missing_piece_globally() {
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
        let selections: VecDeque<PieceSelection> =
            RarestPiecesSelector::priority_pieces_selection(mybitfield, peers_bitfields);

        let mut expected_selection: HashMap<u32, Option<Peer>> = HashMap::new();
        for piece_id in 0..5 {
            expected_selection.insert(piece_id as u32, Some(seeder));
        }

        for selection in &selections {
            match selection.piece_id() {
                0 => assert!(false),
                1..=3 => {
                    assert_eq!(selection.peer(), seeder);
                }
                _ => (),
            }
        }
    }

    #[test]
    pub fn select_rarest_pieces_with_growing_ordrered_rarety_among_three_seeders() {
        let mut peers_bitfields: HashMap<Peer, BitVec> = HashMap::new();
        let bitfield_length = 3;

        // Seeder 1 bitfield is : 111 -> has the rarest piece 2
        let seeder_one: Peer = Peer::from_socket_address(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            0001,
        ));
        let seeder_one_bitfield = BitVec::from_elem(bitfield_length, true);

        // Seeder 2 bitfield is : 110
        let seeder_two: Peer = Peer::from_socket_address(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            0002,
        ));
        let mut seeder_two_bitfield = BitVec::from_elem(bitfield_length, true);
        seeder_two_bitfield.set(2, false);

        // Seeder 3 bitfield is : 100
        let seeder_three: Peer = Peer::from_socket_address(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            0003,
        ));
        let mut seeder_three_bitfield = BitVec::from_elem(bitfield_length, false);
        seeder_three_bitfield.set(0, true);

        peers_bitfields.insert(seeder_one, seeder_one_bitfield);
        peers_bitfields.insert(seeder_two, seeder_two_bitfield);
        peers_bitfields.insert(seeder_three, seeder_three_bitfield);

        let mybitfield: BitVec = BitVec::from_elem(bitfield_length, false);
        let mut selections: VecDeque<PieceSelection> =
            RarestPiecesSelector::priority_pieces_selection(mybitfield, peers_bitfields);

        assert_eq!(
            selections.pop_front().unwrap(),
            PieceSelection::new(2, seeder_one)
        );
        let second_selection = selections.pop_front().unwrap();
        assert!(
            second_selection == PieceSelection::new(1, seeder_one)
                || second_selection == PieceSelection::new(1, seeder_two)
        );
        let third_selection = selections.pop_front().unwrap();
        assert!(
            third_selection == PieceSelection::new(0, seeder_one)
                || third_selection == PieceSelection::new(0, seeder_two)
                || third_selection == PieceSelection::new(0, seeder_three)
        )
    }
}
