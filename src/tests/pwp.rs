#[cfg(test)]
pub mod unittest {
    use crate::pwp::{
        from_bytes, Bitfield, FromBytes, Handshake, Have, Interested, IntoBytes,
        MandatoryBitTorrentMessageFields, MessageType, NotInterested, Piece, Request, Unchoke,
    };
    use bit_vec::BitVec;
    use std::{fs::File, io::Read, path::Path};

    const INFO_ID: [u8; 20] = [
        0x06, 0x71, 0x33, 0xac, 0xe5, 0xdd, 0x0c, 0x50, 0x27, 0xb9, 0x9d, 0xe5, 0xd4, 0xba, 0x51,
        0x28, 0x28, 0x20, 0x8d, 0x5b,
    ];

    const PEER_ID: [u8; 20] = [
        0x2d, 0x42, 0x45, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x6e, 0x9a, 0xb4, 0x40, 0x2c, 0x62, 0x2e,
        0x2e, 0x7a, 0x71, 0x5d, 0x9d,
    ];

    pub fn read_bytes_from(pathfile: &str) -> Vec<u8> {
        let data_filepath = Path::new(pathfile);
        let mut data_file = File::open(data_filepath).unwrap();
        let mut data_buffer: Vec<u8> = Vec::new();
        data_file.read_to_end(&mut data_buffer).unwrap();
        data_buffer
    }

    pub fn path_build_to_pwp_message(file: &str) -> String {
        "samples/peer_wire_protocol-messages/".to_string() + file
    }

    #[test]
    pub fn handshake_message_into_bytes() {
        let expected = read_bytes_from(&path_build_to_pwp_message("handshake.bin"));
        let handshake_bytes = Handshake::new(INFO_ID, PEER_ID);

        assert_eq!(handshake_bytes.into_bytes(), expected);
    }

    #[test]
    pub fn handshake_message_from_bytes() {
        let bytes = read_bytes_from(&path_build_to_pwp_message("handshake.bin"));
        let handshake_to_test = Handshake::from_bytes(&bytes).unwrap().0;

        let expected_handshake = Handshake::new(INFO_ID, PEER_ID);

        assert_eq!(handshake_to_test.pstrlen(), expected_handshake.pstrlen());
        assert_eq!(handshake_to_test.pstr(), expected_handshake.pstr());
        assert_eq!(handshake_to_test.reserved(), expected_handshake.reserved());
        assert_eq!(
            handshake_to_test.info_hash(),
            expected_handshake.info_hash()
        );
        assert_eq!(handshake_to_test.peer_id(), expected_handshake.peer_id());
    }

    #[test]
    pub fn unchoke_message_into_bytes() {
        let expected = read_bytes_from(&path_build_to_pwp_message("unchoke.bin"));
        let unchoke_message = Unchoke::new();

        assert_eq!(unchoke_message.into_bytes(), expected);
    }

    #[test]
    pub fn unchoke_message_from_bytes() {
        let bytes = read_bytes_from(&path_build_to_pwp_message("unchoke.bin"));
        let unchoke_to_test = Unchoke::from_bytes(&bytes).unwrap().0;
        let expected_unchock = Unchoke::new();

        assert_eq!(
            unchoke_to_test.message_length(),
            expected_unchock.message_length()
        );
        assert_eq!(
            unchoke_to_test.message_type(),
            expected_unchock.message_type()
        );
    }

    #[test]
    pub fn interested_message_into_bytes() {
        let expected = read_bytes_from(&path_build_to_pwp_message("interested.bin"));
        let interested_message = Interested::new();

        assert_eq!(interested_message.into_bytes(), expected);
    }

    #[test]
    pub fn interested_message_from_bytes() {
        let bytes = read_bytes_from(&path_build_to_pwp_message("interested.bin"));
        let interested_to_test = Interested::from_bytes(&bytes).unwrap().0;
        let expected_unchock = Interested::new();

        assert_eq!(
            interested_to_test.message_length(),
            expected_unchock.message_length()
        );
        assert_eq!(
            interested_to_test.message_type(),
            expected_unchock.message_type()
        );
    }

    #[test]
    pub fn bitfield_message_into_bytes() {
        let expected = read_bytes_from(&path_build_to_pwp_message("bitfield.bin"));
        let bitfield_message = Bitfield::new(BitVec::from_bytes(&[0xff, 0xe0]));

        assert_eq!(bitfield_message.into_bytes(), expected);
    }

    #[test]
    pub fn bitfield_message_from_bytes() {
        let bytes = read_bytes_from(&path_build_to_pwp_message("bitfield.bin"));
        let bitfield_to_test = Bitfield::from_bytes(&bytes).unwrap().0;
        let expected_bitfield = Bitfield::new(BitVec::from_bytes(&[0xff, 0xe0]));

        assert_eq!(
            bitfield_to_test.message_length(),
            expected_bitfield.message_length()
        );
        assert_eq!(
            bitfield_to_test.message_type(),
            expected_bitfield.message_type()
        );
        assert_eq!(bitfield_to_test.bitfield(), expected_bitfield.bitfield());
    }

    #[test]
    pub fn piece_message_into_bytes() {
        let data = read_bytes_from(&path_build_to_pwp_message("piece_data.bin"));
        let piece_message_to_test = Piece::new(6, 0, data);
        let expected_bytes = read_bytes_from(&path_build_to_pwp_message("piece.bin"));

        assert_eq!(piece_message_to_test.into_bytes(), expected_bytes);
    }

    #[test]
    pub fn piece_message_from_bytes() {
        let piece_bytes = read_bytes_from(&path_build_to_pwp_message("piece.bin"));
        let piece_message_to_test = Piece::from_bytes(&piece_bytes).unwrap().0;

        let data = read_bytes_from(&path_build_to_pwp_message("piece_data.bin"));
        let expected_piece = Piece::new(6, 0, data);

        assert_eq!(
            piece_message_to_test.message_length(),
            expected_piece.message_length()
        );
        assert_eq!(
            piece_message_to_test.message_type(),
            expected_piece.message_type()
        );
        assert_eq!(
            piece_message_to_test.piece_index(),
            expected_piece.piece_index()
        );
        assert_eq!(
            piece_message_to_test.begin_offset_of_piece(),
            expected_piece.begin_offset_of_piece()
        );
        assert_eq!(piece_message_to_test.data(), expected_piece.data());
    }

    #[test]
    pub fn request_message_into_bytes() {
        let request_message_to_test = Request::new(6, 0, 0x4000);
        let expected_bytes = read_bytes_from(&path_build_to_pwp_message("request.bin"));

        assert_eq!(request_message_to_test.into_bytes(), expected_bytes);
    }

    #[test]
    pub fn request_message_from_bytes() {
        let request_bytes = read_bytes_from(&path_build_to_pwp_message("request.bin"));
        let request_message_to_test = Request::from_bytes(&request_bytes).unwrap().0;

        let expected_request = Request::new(6, 0, 0x4000);

        assert_eq!(
            request_message_to_test.message_length(),
            expected_request.message_length()
        );
        assert_eq!(
            request_message_to_test.message_type(),
            expected_request.message_type()
        );
        assert_eq!(
            request_message_to_test.piece_index(),
            expected_request.piece_index()
        );
        assert_eq!(
            request_message_to_test.begin_offset(),
            expected_request.begin_offset()
        );
        assert_eq!(
            request_message_to_test.piece_length(),
            expected_request.piece_length()
        );
    }

    #[test]
    pub fn not_interested_message_into_bytes() {
        let not_interested_message = NotInterested::new();
        let expected_bytes = read_bytes_from(&path_build_to_pwp_message("not_interested.bin"));

        assert_eq!(not_interested_message.into_bytes(), expected_bytes);
    }

    #[test]
    pub fn not_interested_message_from_bytes() {
        let bytes = read_bytes_from(&path_build_to_pwp_message("not_interested.bin"));
        let not_interested_to_test = NotInterested::from_bytes(&bytes).unwrap().0;

        let expected_not_interested = NotInterested::new();

        assert_eq!(
            not_interested_to_test.message_length(),
            expected_not_interested.message_length()
        );
        assert_eq!(
            not_interested_to_test.message_type(),
            expected_not_interested.message_type()
        );
    }

    #[test]
    pub fn have_message_into_bytes() {
        let have_message = Have::new(0x1);
        let expected_bytes = read_bytes_from(&path_build_to_pwp_message("have.bin"));

        assert_eq!(have_message.into_bytes(), expected_bytes);
    }

    #[test]
    pub fn have_message_from_bytes() {
        let bytes = read_bytes_from(&path_build_to_pwp_message("have.bin"));
        let have_to_test = Have::from_bytes(&bytes).unwrap().0;

        let expected_have = Have::new(0x1);

        assert_eq!(
            have_to_test.message_length(),
            expected_have.message_length()
        );
        assert_eq!(have_to_test.message_type(), expected_have.message_type());
        assert_eq!(have_to_test.piece_index(), expected_have.piece_index());
    }

    #[test]
    pub fn identify_bitfield_message_type_from_bytes() {
        let bytes = read_bytes_from(&path_build_to_pwp_message("bitfield.bin"));
        let bitfield_message_type_to_test =
            from_bytes::identity_first_message_type_of(&bytes).unwrap();
        assert_eq!(bitfield_message_type_to_test, MessageType::Bitfield);
    }

    #[test]
    pub fn identify_have_message_type_from_bytes() {
        let bytes = read_bytes_from(&path_build_to_pwp_message("have.bin"));
        let message_type_to_test = from_bytes::identity_first_message_type_of(&bytes).unwrap();
        assert_eq!(message_type_to_test, MessageType::Have);
    }

    #[test]
    pub fn identify_interested_message_type_from_bytes() {
        let bytes = read_bytes_from(&path_build_to_pwp_message("interested.bin"));
        let message_type_to_test = from_bytes::identity_first_message_type_of(&bytes).unwrap();
        assert_eq!(message_type_to_test, MessageType::Interested);
    }

    #[test]
    pub fn identify_not_interested_message_type_from_bytes() {
        let bytes = read_bytes_from(&path_build_to_pwp_message("not_interested.bin"));
        let message_type_to_test = from_bytes::identity_first_message_type_of(&bytes).unwrap();
        assert_eq!(message_type_to_test, MessageType::NotInterested);
    }

    #[test]
    pub fn identify_piece_message_type_from_bytes() {
        let bytes = read_bytes_from(&path_build_to_pwp_message("piece.bin"));
        let message_type_to_test = from_bytes::identity_first_message_type_of(&bytes).unwrap();
        assert_eq!(message_type_to_test, MessageType::Piece);
    }

    #[test]
    pub fn identify_request_message_type_from_bytes() {
        let bytes = read_bytes_from(&path_build_to_pwp_message("request.bin"));
        let message_type_to_test = from_bytes::identity_first_message_type_of(&bytes).unwrap();
        assert_eq!(message_type_to_test, MessageType::Request);
    }

    #[test]
    pub fn identify_unchoke_message_type_from_bytes() {
        let bytes = read_bytes_from(&path_build_to_pwp_message("unchoke.bin"));
        let message_type_to_test = from_bytes::identity_first_message_type_of(&bytes).unwrap();
        assert_eq!(message_type_to_test, MessageType::Unchoke);
    }
}
