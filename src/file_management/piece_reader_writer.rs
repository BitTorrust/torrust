use {
    crate::{Error, Torrent},
    std::{
        fs::{File, OpenOptions},
        os::unix::fs::FileExt,
        path::Path,
    },
};

pub struct PieceReaderWriter {
    torrent: Torrent,
    file: File,
}

impl PieceReaderWriter {
    pub fn new(folder: &Path, torrent: Torrent) -> Result<Self, Error> {
        if !folder.is_dir() {
            return Err(Error::DirectoryDoesNotExist);
        }

        let filename = folder.join(torrent.name().unwrap());
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(filename)
            .map_err(|_| Error::FailedToCreateFile)?;

        Ok(Self { torrent, file })
    }

    pub fn write(&self, piece: u32, piece_offset: u32, data: &[u8]) -> Result<(), Error> {
        let piece_length = self.torrent.piece_length_in_bytes().unwrap();
        let offset = Self::calculate_offset(piece, piece_length, piece_offset);

        self.file
            .write_at(data, offset.into())
            .map_err(|_| Error::FailedToWriteToFile)?;

        Ok(())
    }

    pub fn read(&self, piece: u32, piece_offset: u32) -> Result<Vec<u8>, Error> {
        let piece_length = self.piece_length();
        let offset = Self::calculate_offset(piece, piece_length, piece_offset);

        let mut data = vec![0u8; piece_length as usize];
        self.file
            .read_exact_at(&mut data, offset.into())
            .map_err(|_| Error::FailedToReadFromFile)?;

        Ok(data)
    }

    fn piece_length(&self) -> u32 {
        self.torrent.piece_length_in_bytes().unwrap()
    }

    fn calculate_offset(piece: u32, piece_length: u32, piece_offset: u32) -> u32 {
        piece * piece_length + piece_offset
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Torrent;
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
