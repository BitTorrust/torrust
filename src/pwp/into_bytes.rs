pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
}

impl<T: IntoBytes> IntoBytes for Box<T> {
    fn into_bytes(self) -> Vec<u8> {
        (*self).into_bytes()
    }
}
