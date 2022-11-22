/* Things are always a struct until they become something else */

use crate::{cli::Args, error::Error, torrent::Torrent, BitTorrentStateMachine};
use clap::Parser;

pub struct App {}

impl App {
    pub fn run() -> Result<(), Error> {
        let args = Args::parse();
        let torrent = Torrent::from_file(args.torrent_file())?;
        let directory = args.working_directory();

        BitTorrentStateMachine::run(torrent, directory);

        Ok(())
    }
}
