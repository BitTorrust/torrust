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

    // State machine errors
    NoPeersAvailable,
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
    // For cancel message
    FailedToParseBitTorrentCancelMessagePieceIndex,
    FailedToParseBitTorrentCancelMessageBeginOffset,
    FailedToParseBitTorrentCancelMessagePieceLength,

    // TCP error
    FailedToConnectToPeer,
    TcpSessionDoesNotExist,
    FailedToParseLengthFieldSize,
    FailedToParseReceivedPieceLength,
    FailedToReadFromSocket,
    FailedToCreateTcpListener,
    TcpListenerDoesNotExist,
    FailedToCloneSocketHandle,
    FailedToSetSocketAsNonBlocking,
    FailedToSetSocketWriteTimeout,

    // File management error
    DirectoryDoesNotExist,
    FailedToCreateFile,
    FailedToWriteToFile,
    FailedToReadFromFile,
    UnexpectedBlockSize,
    InvalidWriteOffset,
    InvalidReadOffset,
}
