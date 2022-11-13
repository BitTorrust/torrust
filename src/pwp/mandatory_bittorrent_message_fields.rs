pub trait MandatoryBitTorrentMessageFields {
    fn message_length(&self) -> u32;

    fn message_type(&self) -> u8;
}
