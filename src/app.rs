/* Things are always a struct until they become something else */

use crate::{cli::Args, error::Error, state_machine::StateMachine, torrent::Torrent};
use {clap::Parser, log::LevelFilter, simple_logger::SimpleLogger};

pub struct App {}

impl App {
    pub fn run() -> Result<(), Error> {
        let args = Args::parse();
        if args.debug() {
            Self::init_logger(LevelFilter::Debug);
        } else if args.info() {
            Self::init_logger(LevelFilter::Info);
        }

        let torrent = Torrent::from_file(args.torrent_file())?;
        let directory = args.working_directory();
        let mock_peers = args.mock();
        StateMachine::new(torrent, directory, mock_peers).run();

        Ok(())
    }

    fn init_logger(level: LevelFilter) {
        SimpleLogger::new().with_level(level).init().unwrap();
    }
}
