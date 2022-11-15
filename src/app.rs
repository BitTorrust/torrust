/* Things are always a struct until they become something else */

use crate::{cli::Args, error::Error, torrent::Torrent, BitTorrentStateMachine};
use bendy::decoding::Decoder;
use clap::Parser;
use std::{fs::File, io::Read, path::PathBuf};

pub struct App {}

pub const PEER_ID: [u8; 20] = [
    0xDE, 0xAD, 0xBE, 0xEF, 0xBA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
    0xAA, 0xAA, 0xAA, 0xAD,
];

impl App {
    pub fn run() -> Result<(), Error> {
        let args = Args::parse();
        let torrent = Torrent::from_file(args.torrent_file())?;
        let directory = args.working_directory();

        BitTorrentStateMachine::run(torrent, directory, PEER_ID);

        Ok(())
    }
}
