mod torrent;
pub use torrent::Torrent;

mod error;
use error::Error;

#[cfg(test)]
mod tests;

fn main() -> Result<(), Error> {
    Ok(())
}
