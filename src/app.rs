/* Things are always a struct until they become something else */

use crate::{cli::Args, error::Error, state_machine::StateMachine, torrent::Torrent};
use {clap::Parser, log::LevelFilter, simple_logger::SimpleLogger};

pub struct App {}

impl App {
    pub fn run() -> Result<(), Error> {
        SimpleLogger::new()
            .with_level(LevelFilter::Debug)
            .init()
            .unwrap();

        let args = Args::parse();
        let torrent = Torrent::from_file(args.torrent_file())?;
        let directory = args.working_directory();

        //BitTorrentStateMachine::run(torrent, directory);
        StateMachine::new(torrent, directory).run();

        Ok(())
    }
}
