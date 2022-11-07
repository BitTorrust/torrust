mod torrent;
pub use torrent::Torrent;
mod error;
pub use error::Error;

mod http;

mod pwp;

mod cli;

use clap::Parser;
use cli::Args;
use std::fs;

#[cfg(test)]
mod tests;

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let torrent_file_content = fs::read(args.torrent_file()).unwrap();

    if args.info() {
        println!("INFO flag set");
    }

    if args.debug() {
        println!("DEBUG flag set");
    }

    // Main program
    println!("\nWorking directory: {:?}\n", args.working_directory());
    println!("Torrent file content: {:?}", torrent_file_content);

    Ok(())
}
