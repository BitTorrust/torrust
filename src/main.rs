mod torrent;
pub use torrent::Torrent;

mod error;
pub use error::Error;

mod file_management;
pub use file_management::BlockReaderWriter;

mod app;
use app::App;

mod cli;
mod http;
mod pwp;
mod tcp;

#[cfg(test)]
mod tests;

fn main() -> Result<(), Error> {
    App::run()
}
