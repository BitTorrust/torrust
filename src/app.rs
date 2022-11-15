/* Things are always a struct until they become something else */

use crate::{cli::Args, error::Error, torrent::Torrent, BitTorrentStateMachine};
use bendy::decoding::Decoder;
use clap::Parser;
use std::{fs::File, io::Read, path::PathBuf};

pub struct App {}

impl App {
    pub fn run() -> Result<(), Error> {
        let args = Args::parse();
        let torrent = Self::parse_torrent(args.torrent_file())?;
        let directory = args.working_directory();

        BitTorrentStateMachine::run(torrent, directory);

        Ok(())
    }

    fn parse_torrent(torrent_filepath: &PathBuf) -> Result<Torrent, Error> {
        let mut torrent_file =
            File::open(torrent_filepath).map_err(|_| Error::FailedToOpenTorrentFile)?;

        let mut torrent_file_content = Vec::new();
        torrent_file
            .read_to_end(&mut torrent_file_content)
            .map_err(|_e| Error::FailedToReadTorrentFile)?;

        let mut bencode_decoder = Decoder::new(&torrent_file_content);
        let torrent = Torrent::from_bencode(&mut bencode_decoder)
            .map_err(|_| Error::FailedToDecodeBencodeData)?;

        Ok(torrent)
    }
}
