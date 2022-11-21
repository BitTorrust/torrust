#[derive(Debug)]
pub enum Error {
    // Torrent file parsing error
    FailedToOpenTorrentFile,
    FailedToReadTorrentFile,
    FailedToParseTorrentFile,
    FailedToGetRawBytesFromInfoDict,
    TotalPiecesLengthNotFoundDuringParsing,
    SinglePieceLengthNotFoundDuringParsing,
    AnnounceBytesCannotBeConvertedToString,
    HashedInfoDictCannotConvertToTwentyBytesVec,
    PieceLengthStringCannotBeConvertedToInteger,
    LengthStringCannotBeConvertedToInteger,
    NameBytesCannotBeConvertedToString,

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

    // Handshake message error
    FailedToParseBitTorrentHandshakeProtocolNameField,
    FailedToParseBitTorrentHandshakeReservedField,
    FailedToParseBitTorrentHandshakeInfoHashField,
    FailedToParseBitTorrentHandshakePeerIDField,
    // Peer wire protocol message parsing error
    FailedToParseBitTorrentMessageLength,
    MessageLengthDoesNotMatchWithExpectedOne,
    BytesArrayTooShort,
    MessageTypeDoesNotMatchWithExpectedOne,
    BytesArrayTooShortToContrainMessageFields,
    FailedToFindTheMessageTypeOfRawBytes,
    // For request message
    FailedToParseBitTorrentRequestMessagePieceIndex,
    FailedToParseBitTorrentRequestMessageBeginOffset,
    FailedToParseBitTorrentRequestMessagePieceLength,
    // For piece message
    FailedToParseBitTorrentPieceMessagePieceIndex,
    FailedToParseBitTorrentPieceMessageBeginOffset,
    FailedToParseBitTorrentPieceMessagePieceLength,
    // For have message
    FailedToParseBitTorrentHaveMessagePieceIndex,

    // TCP error
    FailedToConnectToPeer,
    TcpSessionDoesNotExist,
    FailedToParseReceivedBitfieldLength,
    FailedToParseReceivedPieceLength,
    FailedToReadFromSocket,

    // File management error
    DirectoryDoesNotExist,
    FailedToCreateFile,
    FailedToWriteToFile,
    FailedToReadFromFile,
    UnexpectedBlockSize,
    InvalidWriteOffset,
    InvalidReadOffset,
}
