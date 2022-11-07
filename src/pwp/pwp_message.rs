pub trait ByteConvertable {
    fn into_bytes(self) -> Vec<u8>;
}
