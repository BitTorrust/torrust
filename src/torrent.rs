use crate::Error;
use bendy::decoding::{Decoder, DictDecoder, Object};
use sha1::{Digest, Sha1};
use std::{fs::File, io::Read, path::Path, str::FromStr};

#[derive(Debug)]
pub struct Torrent {
    /// URL of the tracker
    announce: String,
    /// number of bytes in each piece
    piece_length_in_bytes: u32,
    /// pieces number calculted with total_length_in_bytes and piece_length_in_bytes
    number_of_pieces: u32,
    /// length of file
    total_length_in_bytes: u32,
    /// the filename
    name: String,
    /// a 160-bit (20-byte)
    info_hash: [u8; 20],
}

impl Torrent {
    pub fn announce(&self) -> &String {
        &self.announce
    }

    pub fn piece_length_in_bytes(&self) -> u32 {
        self.piece_length_in_bytes
    }

    pub fn number_of_pieces(&self) -> u32 {
        self.number_of_pieces
    }

    pub fn total_length_in_bytes(&self) -> u32 {
        self.total_length_in_bytes
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn info_hash(&self) -> [u8; 20] {
        self.info_hash.clone()
    }

    pub fn decode_dict(&mut self, dict: &mut DictDecoder) -> Result<(), Error> {
        while let Ok(Some(pair)) = dict.next_pair() {
            let key = String::from_utf8(pair.0.to_vec()).unwrap();
            match key.as_str() {
                "announce" => {
                    self.announce = match pair.1 {
                        Object::Bytes(byte) => String::from_utf8(byte.to_vec())
                            .map_err(|_| Error::AnnounceBytesCannotBeConvertedToString)?,
                        _ => return Err(Error::BencodeObjectHasUnexpectedType),
                    }
                }
                "info" => match pair.1 {
                    Object::Dict(mut info_dict) => {
                        self.decode_dict(&mut info_dict)?;

                        // hash the info dictionnary
                        let mut hasher = Sha1::new();
                        let raw_bytes = info_dict
                            .into_raw()
                            .map_err(|_| Error::FailedToGetRawBytesFromInfoDict)?;
                        hasher.update(raw_bytes);
                        let hash_vec = hasher.finalize().to_vec();
                        self.info_hash = hash_vec
                            .try_into()
                            .clone()
                            .map_err(|_| Error::HashedInfoDictCannotConvertToTwentyBytesVec)?;
                    }
                    _ => (),
                },
                "piece length" => {
                    self.piece_length_in_bytes = match pair.1 {
                        Object::Integer(string) => u32::from_str(string)
                            .map_err(|_| Error::PieceLengthStringCannotBeConvertedToInteger)?,
                        _ => return Err(Error::BencodeObjectHasUnexpectedType),
                    }
                }
                "length" => {
                    self.total_length_in_bytes = match pair.1 {
                        Object::Integer(string) => u32::from_str(string)
                            .map_err(|_| Error::LengthStringCannotBeConvertedToInteger)?,
                        _ => return Err(Error::BencodeObjectHasUnexpectedType),
                    }
                }
                "name" => {
                    self.name = match pair.1 {
                        Object::Bytes(byte) => String::from_utf8(byte.to_vec())
                            .map_err(|_| Error::NameBytesCannotBeConvertedToString)?,
                        _ => return Err(Error::BencodeObjectHasUnexpectedType),
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }

    pub fn from_file(filepath: &Path) -> Result<Torrent, Error> {
        let mut file = File::open(filepath).map_err(|_| Error::FailedToOpenTorrentFile)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|_| Error::FailedToReadTorrentFile)?;

        let mut bencode_decoder = Decoder::new(&buffer);
        let torrent = Torrent::from_bencode(&mut bencode_decoder)?;

        Ok(torrent)
    }

    pub fn from_bencode(bencode_decoder: &mut Decoder) -> Result<Torrent, Error> {
        let mut torrent_result = Torrent {
            announce: String::from(""),
            piece_length_in_bytes: 0,
            number_of_pieces: 0,
            total_length_in_bytes: 0,
            name: String::from(""),
            info_hash: [0; 20],
        };

        let maybe_bencode_object = bencode_decoder
            .next_object()
            .map_err(|_| Error::FailedToParseTorrentFile)?;

        match maybe_bencode_object {
            Some(Object::Dict(mut dict_decoder)) => {
                torrent_result.decode_dict(&mut dict_decoder)?;
            }
            None => (), // EOF
            _ => (),
        };

        torrent_result.number_of_pieces = div_ceil(
            torrent_result.total_length_in_bytes(),
            torrent_result.piece_length_in_bytes(),
        );

        Ok(torrent_result)
    }
}

pub fn div_ceil(a: u32, b: u32) -> u32 {
    a / b + if a % b == 0 { 0 } else { 1 }
}
