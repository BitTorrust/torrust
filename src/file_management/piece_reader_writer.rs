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

    pub fn piece_length(&self) -> u32 {
        self.torrent.piece_length_in_bytes().unwrap()
    }

    pub fn calculate_offset(piece: u32, piece_length: u32, piece_offset: u32) -> u32 {
        piece * piece_length + piece_offset
    }
}
