#[cfg(test)]
mod test {
    use crate::BlockReaderWriter;
    use std::path::Path;

    #[test]
    fn read_and_write_pieces() {
        let block_length = BlockReaderWriter::BIT_TORRENT_BLOCK_SIZE as u32;
        let extra_bytes = 15 * 1024;
        let filesize = block_length * 6 + extra_bytes;

        let filename = Path::new("with_offset.jpg");
        let file = BlockReaderWriter::new(filename, block_length * 2, filesize as usize).unwrap();

        let piece_1_block_1 = vec![0xBB; block_length as usize];
        file.write(1, 0, &piece_1_block_1).unwrap();

        let piece_1_block_2 = vec![0xBB; block_length as usize];
        file.write(1, block_length, &piece_1_block_2).unwrap();

        let piece_0_block_1 = vec![0xAA; block_length as usize];
        file.write(0, 0, &piece_0_block_1).unwrap();

        let piece_0_block_2 = vec![0xAA; block_length as usize];
        file.write(0, block_length, &piece_0_block_2).unwrap();

        let piece_2_block_1 = vec![0xCC; block_length as usize];
        file.write(2, 0, &piece_2_block_1).unwrap();

        let piece_2_block_2 = vec![0xCC; block_length as usize];
        file.write(2, block_length, &piece_2_block_2).unwrap();

        let extra_bytes = vec![0xDD; extra_bytes as usize];
        file.write(3, 0, &extra_bytes).unwrap();

        assert_eq!(file.read(0, 0).unwrap(), piece_0_block_1);
        assert_eq!(file.read(0, block_length).unwrap(), piece_0_block_2);

        assert_eq!(file.read(1, 0).unwrap(), piece_1_block_1);
        assert_eq!(file.read(1, block_length).unwrap(), piece_1_block_2);

        assert_eq!(file.read(2, 0).unwrap(), piece_2_block_1);
        assert_eq!(file.read(2, block_length).unwrap(), piece_2_block_2);

        assert_eq!(file.read(3, 0).unwrap(), extra_bytes);
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
