#[derive(Debug)]
pub enum Error {
    FailedToOpenTorrentFile,
    FailedToReadTorrentFile,
    FailedToParserTorrentFile,
}
