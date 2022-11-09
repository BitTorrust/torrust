use crate::pwp::IntoBytes;

pub struct Handshake {
    /// string length of <pstr>, as a single raw byte
    pstrlen: u8,

    /// pstr: string identifier of the protocol
    pstr: String,

    /// reserved: eight (8) reserved bytes. All current implementations use
    /// all zeroes.
    reserved: [u8; 8],

    /// info_hash: 20-byte SHA1 hash of the info key in the metainfo file.
    info_hash: [u8; 20],

    /// peer_id: 20-byte string used as a unique ID for the client.
    peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        let pstr = "BitTorrent protocol".to_string();
        let pstrlen: u8 = pstr.len() as u8;
        let reserved = [0; 8];

        Self {
            pstr,
            pstrlen,
            reserved,
            info_hash,
            peer_id,
        }
    }
}

impl IntoBytes for Handshake {
    fn into_bytes(self) -> Vec<u8> {
        let mut serialized_message = Vec::new();

        serialized_message.push(self.pstrlen);
        serialized_message.extend_from_slice(self.pstr.as_bytes());
        serialized_message.extend_from_slice(&self.reserved);
        serialized_message.extend_from_slice(&self.info_hash);
        serialized_message.extend_from_slice(&self.peer_id);

        serialized_message
    }
}
