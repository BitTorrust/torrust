mod torrent;
pub use torrent::Torrent;
mod error;
pub use error::Error;

use crate::app::App;

mod app;
mod cli;
mod file_management;
mod http;
mod pwp;

#[cfg(test)]
mod tests;

fn main() -> Result<(), Error> {
    App::run()
}
