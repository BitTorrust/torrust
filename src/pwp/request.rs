#[derive(Debug)]
pub struct Request {
    message_length: u32,
    message_type: u8,
    piece_index: u32,
    begin_offset: u32,
    piece_length: u32,
}

impl Request {
    pub fn new() -> Self {
        unimplemented!()
    }
}
