#[derive(Debug)]
pub enum Error {
    FailedToOpenTorrentFile,
    FailedToReadTorrentFile,
    FailedToParserTorrentFile,
}

mod torrent;
pub use torrent::Torrent;

#[cfg(test)]
mod tests;
fn main() -> Result<(), Error> {
    Ok(())
}
