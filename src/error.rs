#[derive(Debug)]
pub enum Error {
    FailedToOpenTorrentFile,
    FailedToReadTorrentFile,
    FailedToParseTorrentFile,
    FailedToGetRawBytesFromInfoDict,
    TotalPiecesLengthNotFoundDuringParsing,
    SinglePieceLengthNotFoundDuringParsing,
}
