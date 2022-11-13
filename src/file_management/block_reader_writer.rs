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
    file_size: usize,
    file: File,
}

impl BlockReaderWriter {
    pub const BIT_TORRENT_BLOCK_SIZE: usize = 16 * 1024;

    pub fn new(filepath: &Path, piece_length: u32, file_size: usize) -> Result<Self, Error> {
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(filepath)
            .map_err(|_| Error::FailedToCreateFile)?;

        Ok(Self {
            piece_length,
            file,
            file_size,
        })
    }

    pub fn write(&self, piece_index: u32, piece_offset: u32, data: &[u8]) -> Result<(), Error> {
        if data.len() > Self::BIT_TORRENT_BLOCK_SIZE as usize {
            return Err(Error::UnexpectedBlockSize);
        }

        let offset = Self::calculate_offset(piece_index, self.piece_length(), piece_offset);

        self.file
            .write_at(data, offset.into())
            .map_err(|_| Error::FailedToWriteToFile)?;

        Ok(())
    }

    pub fn read(&self, piece_index: u32, piece_offset: u32) -> Result<Vec<u8>, Error> {
        let offset = Self::calculate_offset(piece_index, self.piece_length(), piece_offset);
        let bytes_to_read = self.bytes_to_read(offset);

        let mut data = vec![0u8; bytes_to_read];
        self.file
            .read_exact_at(&mut data, offset as u64)
            .map_err(|_| Error::FailedToReadFromFile)?;

        Ok(data)
    }

    pub fn piece_length(&self) -> u32 {
        self.piece_length
    }

    pub fn calculate_offset(piece: u32, piece_length: u32, piece_offset: u32) -> u32 {
        piece * piece_length + piece_offset
    }

    fn bytes_to_read(&self, offset: u32) -> usize {
        if (offset as usize + Self::BIT_TORRENT_BLOCK_SIZE) > self.file_size {
            self.file_size - offset as usize
        } else {
            Self::BIT_TORRENT_BLOCK_SIZE
        }
    }
}
