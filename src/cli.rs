use clap::{ArgAction, Parser};
use std::path::PathBuf;

/// A very humble Torrent client made with all our effort
#[derive(Parser, Debug)]
pub struct Args {
    /// The .torrent file path
    torrent_file: PathBuf,

    /// The download path to store/upload the file described in .torrent
    working_directory: PathBuf,

    /// Gives network peers information (bittorrent application, address IP, port, download/upload piece state)
    #[arg(short, long, action = ArgAction::SetTrue)]
    info: bool,

    /// Print minimal debug info
    #[arg(short, long,  action = ArgAction::SetTrue)]
    debug: bool,
}

impl Args {
    pub fn torrent_file(&self) -> &PathBuf {
        &self.torrent_file
    }

    pub fn working_directory(&self) -> &PathBuf {
        &self.working_directory
    }

    pub fn info(&self) -> bool {
        self.info
    }

    pub fn debug(&self) -> bool {
        self.debug
    }
}
