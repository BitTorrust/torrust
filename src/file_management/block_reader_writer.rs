use {
    crate::Error,
    std::{
        fs::{File, OpenOptions},
        os::unix::fs::FileExt,
        path::Path,
    },
};

pub struct BlockReaderWriter {
    piece_length: u32,
    file: File,
}

impl BlockReaderWriter {
    pub const BIT_TORRENT_BLOCK_SIZE: usize = 16 * 1024;

    pub fn new(filepath: &Path, piece_length: u32) -> Result<Self, Error> {
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(filepath)
            .map_err(|_| Error::FailedToCreateFile)?;

        Ok(Self { piece_length, file })
    }

    pub fn write_block(&self, piece: u32, piece_offset: u32, data: &[u8]) -> Result<(), Error> {
        if data.len() != Self::BIT_TORRENT_BLOCK_SIZE {
            return Err(Error::UnexpectedBlockSize);
        }

        let offset = Self::calculate_offset(piece, self.piece_length(), piece_offset);

        self.file
            .write_at(data, offset.into())
            .map_err(|_| Error::FailedToWriteToFile)?;

        Ok(())
    }

    pub fn read_block(&self, piece: u32, piece_offset: u32) -> Result<Vec<u8>, Error> {
        let offset = Self::calculate_offset(piece, self.piece_length(), piece_offset);

        let mut data = vec![0u8; Self::BIT_TORRENT_BLOCK_SIZE];
        self.file
            .read_exact_at(&mut data, offset.into())
            .map_err(|_| Error::FailedToReadFromFile)?;

        Ok(data)
    }

    pub fn piece_length(&self) -> u32 {
        self.piece_length
    }

    pub fn calculate_offset(piece: u32, piece_length: u32, piece_offset: u32) -> u32 {
        piece * piece_length + piece_offset
    }
}
