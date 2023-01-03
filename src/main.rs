mod torrent;
pub use torrent::{div_ceil, Torrent};

mod error;
pub use error::Error;

mod file_management;
pub use file_management::BlockReaderWriter;

mod app;
use app::App;

mod cli;
mod http;
mod pwp;
pub use pwp::*;
mod tcp;

mod state_machine;

mod adaptative_wait;

mod pieces_selection;

#[cfg(test)]
mod tests;

fn main() -> Result<(), Error> {
    App::run()
}
