#[derive(Debug)]
pub enum Error {
    // Torrent file parsing error
    FailedToOpenTorrentFile,
    FailedToReadTorrentFile,
    FailedToParseTorrentFile,
    FailedToGetRawBytesFromInfoDict,
    TotalPiecesLengthNotFoundDuringParsing,
    SinglePieceLengthNotFoundDuringParsing,

    // HTTP announce error
    FailedToParseUrl,
    BencodeObjectHasUnexpectedType,
    UnexpectedResponseFromTracker,
    TrackerFailureMessageContainsNonUtf8Characters,
    InvalidURLAddress,
    TrackerHostNotProvided,
    TrackerPortNotProvided,
    FailedToDecodeBencodeData,
    TrackerConnectionNotPossible,
    // Peer wire protocol message parsing error
    FailedToParseBitTorrentMessageLength,
    MessageLengthDoesNotMatchWithExpectedOne,
    BytesArrayTooShort,
    MessageTypeDoesNotMatchWithExpectedOne,
    // For request message
    FailedToParseBitTorrentRequestMessagePieceIndex,
    FailedToParseBitTorrentRequestMessageBeginOffset,
    FailedToParseBitTorrentRequestMessagePieceLength,
    // For piece message
    FailedToParseBitTorrentPieceMessagePieceIndex,
    FailedToParseBitTorrentPieceMessageBeginOffset,
    FailedToParseBitTorrentPieceMessagePieceLength,
    
    // TCP error
    FailedToConnectToPeer,

    // File management error
    DirectoryDoesNotExist,
    FailedToCreateFile,
    FailedToWriteToFile,
    FailedToReadFromFile,
    UnexpectedBlockSize,
    InvalidWriteOffset,
    InvalidReadOffset,
}
