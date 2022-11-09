#[cfg(test)]
mod test {
    use crate::{PieceReaderWriter, Torrent};
    use std::path::Path;

    #[test]
    fn read_and_write_pieces() {
        let torrent = Torrent::from_file(Path::new("./samples/iceberg.jpg.torrent")).unwrap();
        let torrent_writer = PieceReaderWriter::new(Path::new("."), torrent).unwrap();
        let piece_length = torrent_writer.piece_length() as usize;

        let second_piece = vec![0xBB; piece_length];
        torrent_writer.write(1, 0, &second_piece).unwrap();

        let first_piece = vec![0xAA; piece_length];
        torrent_writer.write(0, 0, &first_piece).unwrap();

        let third_piece = vec![0xCC; piece_length];
        torrent_writer.write(2, 0, &third_piece).unwrap();

        assert_eq!(torrent_writer.read(0, 0).unwrap(), first_piece.to_vec());
        assert_eq!(torrent_writer.read(1, 0).unwrap(), second_piece.to_vec());
        assert_eq!(torrent_writer.read(2, 0).unwrap(), third_piece.to_vec());
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
