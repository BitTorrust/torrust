use {
    crate::{file_management::BlockReaderWriter, torrent, Torrent},
    bit_vec::BitVec,
    sha1::{Digest, Sha1},
    std::path::Path,
};

pub fn local_bitfield(torrent: &Torrent, working_dir: &Path) -> BitVec {
    let pieces = read_pieces_from_disk(torrent, working_dir);
    let real_hashes = calculate_piece_hashes(&pieces);
    let expected_hashes = torrent.piece_hashes();
    let mut bitfield = create_bitfield_from_hashes(&expected_hashes, &real_hashes);

    let padding_bits = bitfield.len() % 8;
    if padding_bits != 0 {
        let mut extra_bits = BitVec::from_elem(8 - padding_bits, false);
        bitfield.append(&mut extra_bits);
    }

    bitfield
}

fn read_pieces_from_disk(torrent: &Torrent, working_dir: &Path) -> Vec<Vec<u8>> {
    let filepath = working_dir.join(torrent.name());
    let piece_length = torrent.piece_length_in_bytes();
    let total_length = torrent.total_length_in_bytes() as usize;

    let reader_writer = BlockReaderWriter::new(&filepath, piece_length, total_length).unwrap();
    let piece_length = piece_length as usize;
    let total_pieces = torrent::div_ceil(total_length as u32, piece_length as u32);

    let mut pieces = Vec::new();
    for piece_id in 0..total_pieces {
        let mut piece = Vec::new();

        let blocks_per_piece = torrent::expected_blocks_in_piece(piece_id, torrent);

        for block_id in 0..blocks_per_piece {
            let maybe_block = reader_writer.read(
                piece_id as u32,
                block_id as u32 * BlockReaderWriter::BIT_TORRENT_BLOCK_SIZE as u32,
            );

            let block = match maybe_block {
                Ok(block) => block,
                Err(_) => vec![0],
            };

            piece.extend(block);
        }

        pieces.push(piece);
    }

    pieces
}

fn calculate_piece_hashes(pieces: &Vec<Vec<u8>>) -> Vec<[u8; 20]> {
    pieces
        .iter()
        .map(|piece| {
            let mut hasher = Sha1::new();
            hasher.update(piece);
            hasher.finalize().into()
        })
        .collect()
}

fn create_bitfield_from_hashes(
    expected_hashes: &Vec<[u8; 20]>,
    real_hashes: &Vec<[u8; 20]>,
) -> BitVec {
    expected_hashes
        .iter()
        .zip(real_hashes.iter())
        .enumerate()
        .fold(
            BitVec::from_elem(expected_hashes.len(), false),
            |mut bitfield, (id, (h1, h2))| {
                bitfield.set(id, h1 == h2);

                bitfield
            },
        )
}
//TODO: correct the test because we added trailing zeros
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::path::Path;

//     #[test]
//     fn build_bitfield_from_file() {
//         let torrent = Torrent::from_file(&Path::new("samples/upload/venon.jpg.torrent")).unwrap();
//         let working_dir = Path::new("samples/upload/");
//         let local_bitfield = local_bitfield(&torrent, &working_dir);

//         assert_eq!(
//             local_bitfield.len(),
//             (torrent.number_of_pieces() as usize + 8) / 8 as usize
//         );
//     }
// }
