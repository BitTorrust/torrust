mod cli;

use clap::Parser;
use cli::Args;
use std::fs;

fn main() {
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
}
