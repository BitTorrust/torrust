#[cfg(test)]
mod test {
    use crate::BlockReaderWriter;
    use std::path::Path;

    #[test]
    fn read_and_write_pieces() {
        let file = BlockReaderWriter::new(Path::new("with_offset.jpg"), 32 * 1024).unwrap();
        let block_length = BlockReaderWriter::BIT_TORRENT_BLOCK_SIZE;

        let piece_1_block_1 = vec![0xBB; block_length];
        file.write_block(1, 0, &piece_1_block_1).unwrap();

        let piece_1_block_2 = vec![0xBB; block_length];
        file.write_block(1, 0x4000, &piece_1_block_2).unwrap();

        let piece_0_block_1 = vec![0xAA; block_length];
        file.write_block(0, 0, &piece_0_block_1).unwrap();

        let piece_0_block_2 = vec![0xAA; block_length];
        file.write_block(0, 0x4000, &piece_0_block_2).unwrap();

        let piece_2_block_1 = vec![0xCC; block_length];
        file.write_block(2, 0, &piece_2_block_1).unwrap();

        let piece_2_block_2 = vec![0xCC; block_length];
        file.write_block(2, 0x4000, &piece_2_block_2).unwrap();

        assert_eq!(file.read_block(0, 0x0000).unwrap(), piece_0_block_1);
        assert_eq!(file.read_block(0, 0x4000).unwrap(), piece_0_block_2);

        assert_eq!(file.read_block(1, 0x0000).unwrap(), piece_1_block_1);
        assert_eq!(file.read_block(1, 0x4000).unwrap(), piece_1_block_2);

        assert_eq!(file.read_block(2, 0x0000).unwrap(), piece_2_block_1);
        assert_eq!(file.read_block(2, 0x4000).unwrap(), piece_2_block_2);
    }

    #[test]
    fn calculate_offset() {
        let offset = BlockReaderWriter::calculate_offset(0, 32 * 1024, 0);
        assert_eq!(offset, 0);

        let offset = BlockReaderWriter::calculate_offset(0, 32 * 1024, 16384);
        assert_eq!(offset, 16384);

        let offset = BlockReaderWriter::calculate_offset(1, 32 * 1024, 0);
        assert_eq!(offset, 32768);

        let offset = BlockReaderWriter::calculate_offset(1, 32 * 1024, 16384);
        assert_eq!(offset, 49152);
    }
}
