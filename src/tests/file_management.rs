#[cfg(test)]
mod test {
    use crate::PieceReaderWriter;
    use std::path::Path;

    #[test]
    fn read_and_write_pieces() {
        let file = PieceReaderWriter::new(Path::new("test.jpg"), 32 * 1024).unwrap();
        let piece_length = file.piece_length() as usize;

        let second_piece = vec![0xBB; piece_length];
        file.write(1, 0, &second_piece).unwrap();

        let first_piece = vec![0xAA; piece_length];
        file.write(0, 0, &first_piece).unwrap();

        let third_piece = vec![0xCC; piece_length];
        file.write(2, 0, &third_piece).unwrap();

        assert_eq!(file.read(0, 0).unwrap(), first_piece.to_vec());
        assert_eq!(file.read(1, 0).unwrap(), second_piece.to_vec());
        assert_eq!(file.read(2, 0).unwrap(), third_piece.to_vec());
    }

    #[test]
    fn calculate_offset() {
        let offset = PieceReaderWriter::calculate_offset(0, 32 * 1024, 0);
        assert_eq!(offset, 0);

        let offset = PieceReaderWriter::calculate_offset(0, 32 * 1024, 16384);
        assert_eq!(offset, 16384);

        let offset = PieceReaderWriter::calculate_offset(1, 32 * 1024, 0);
        assert_eq!(offset, 32768);

        let offset = PieceReaderWriter::calculate_offset(1, 32 * 1024, 16384);
        assert_eq!(offset, 49152);
    }
}
