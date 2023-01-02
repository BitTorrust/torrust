use crate::pwp::{FromBytes, IntoBytes};
use crate::Error;

#[derive(Debug)]
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
    pub const HANDSHAKE_MESSAGE_LENGTH_FIELD_SIZE: usize = 1;
    pub const HANDSHAKE_MIN_MESSAGE_SIZE: usize =
        Handshake::HANDSHAKE_MESSAGE_LENGTH_FIELD_SIZE + 8 + 20 + 20; // without pstr taken into account
    pub const BITTORRENT_VERSION_1_PROTOCOL_NAME: &'static str = "BitTorrent protocol";
    pub const BITTORRENT_VERSION_1_PROTOCOL_NAME_LENGTH: u8 =
        Handshake::BITTORRENT_VERSION_1_PROTOCOL_NAME.len() as u8;
    pub const HANDSHAKE_VERSION_1_MESSAGE_LENGTH: usize = Handshake::HANDSHAKE_MIN_MESSAGE_SIZE
        + Handshake::BITTORRENT_VERSION_1_PROTOCOL_NAME_LENGTH as usize;

    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        let pstr = Handshake::BITTORRENT_VERSION_1_PROTOCOL_NAME.to_string();
        let pstrlen: u8 = pstr.len() as u8;
        let reserved = [0; 8];

        Self {
            pstrlen,
            pstr,
            reserved,
            info_hash,
            peer_id,
        }
    }

    pub fn pstrlen(&self) -> u8 {
        self.pstrlen
    }

    pub fn pstr(&self) -> &String {
        &self.pstr
    }

    pub fn reserved(&self) -> [u8; 8] {
        self.reserved
    }

    pub fn info_hash(&self) -> [u8; 20] {
        self.info_hash
    }

    pub fn peer_id(&self) -> [u8; 20] {
        self.peer_id
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

impl FromBytes for Handshake {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), Error> {
        if bytes.len() < Handshake::HANDSHAKE_MIN_MESSAGE_SIZE {
            return Err(Error::BytesArrayTooShort);
        }

        let pstrlen = bytes[0]; // expecting 19 for "BitTorrent protocol" string

        let pstr_ending_offset = 1 + pstrlen as usize;
        let pstr = String::from_utf8(bytes[1..pstr_ending_offset].to_vec())
            .map_err(|_| Error::FailedToParseBitTorrentHandshakeProtocolNameField)?;

        let reserved: [u8; 8] = bytes[pstr_ending_offset..pstr_ending_offset + 8]
            .try_into()
            .map_err(|_| Error::FailedToParseBitTorrentHandshakeReservedField)?;

        let info_hash = bytes[pstr_ending_offset + 8..pstr_ending_offset + 28]
            .try_into()
            .map_err(|_| Error::FailedToParseBitTorrentHandshakeInfoHashField)?;

        let peer_id = bytes[pstr_ending_offset + 28..pstr_ending_offset + 48]
            .try_into()
            .map_err(|_| Error::FailedToParseBitTorrentHandshakePeerIDField)?;

        Ok((
            Self {
                pstrlen,
                pstr,
                reserved,
                info_hash,
                peer_id,
            },
            (pstrlen as usize + Handshake::HANDSHAKE_MIN_MESSAGE_SIZE) as usize,
        ))
    }
}
