/* Things are always a struct until they become something else */

use crate::{cli::Args, error::Error, state_machine::StateMachine, torrent::Torrent};
use {clap::Parser, log::LevelFilter, simple_logger::SimpleLogger};

pub struct App {}

impl App {
    pub fn run() -> Result<(), Error> {
        let args = Args::parse();
        if args.debug() {
            Self::init_logger();
        }

        let torrent = Torrent::from_file(args.torrent_file())?;
        let directory = args.working_directory();
        StateMachine::new(torrent, directory).run();

        Ok(())
    }

    fn init_logger() {
        SimpleLogger::new()
            .with_level(LevelFilter::Debug)
            .init()
            .unwrap();
    }
}
