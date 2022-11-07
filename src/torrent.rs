use crate::Error;
use bendy::decoding::{Decoder, DictDecoder, Object};
use sha1::{Digest, Sha1};
use std::str::FromStr;

#[derive(Debug)]
pub struct Torrent {
    announce: Option<String>,
    piece_length_in_bytes: Option<u32>,
    number_of_pieces: Option<u32>,
    total_length_in_bytes: Option<u32>,
    name: Option<String>,
    info_hash: Option<String>,
}

impl Torrent {
    pub fn announce(&self) -> Option<&String> {
        self.announce.as_ref()
    }

    pub fn piece_length_in_bytes(&self) -> Option<u32> {
        self.piece_length_in_bytes
    }

    pub fn number_of_pieces(&self) -> Option<u32> {
        self.number_of_pieces
    }

    pub fn total_length_in_bytes(&self) -> Option<u32> {
        self.total_length_in_bytes
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn info_hash(&self) -> Option<&String> {
        self.info_hash.as_ref()
    }

    pub fn decode_dict(&mut self, dict: &mut DictDecoder) -> Result<(), Error> {
        while let Ok(Some(pair)) = dict.next_pair() {
            let key = String::from_utf8(pair.0.to_vec()).unwrap();
            match key.as_str() {
                "announce" => {
                    self.announce = match pair.1 {
                        Object::Bytes(byte) => Some(String::from_utf8(byte.to_vec()).unwrap()),
                        _ => None,
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
                        println!(
                            "info_dict's bytes: {:?}",
                            String::from_utf8(raw_bytes.to_vec())
                        );
                        hasher.update(raw_bytes);
                        println!("{:?}", &hasher.finalize().to_vec());
                        // self.info_hash = Some(
                        //     String::from(
                        //             encode(
                        //                 std::str::from_utf8(&
                        //                 hasher.finalize().to_vec()
                        //             ).unwrap()
                        //         )
                        //     )
                        // );
                        // println!("info_hash {:?}", self.info_hash);
                    }
                    _ => (),
                },
                "piece length" => {
                    self.piece_length_in_bytes = match pair.1 {
                        Object::Integer(string) => Some(u32::from_str(string).unwrap()),
                        _ => None,
                    }
                }
                "length" => {
                    self.total_length_in_bytes = match pair.1 {
                        Object::Integer(string) => Some(u32::from_str(string).unwrap()),
                        _ => None,
                    }
                }
                "name" => {
                    self.name = match pair.1 {
                        Object::Bytes(byte) => Some(String::from_utf8(byte.to_vec()).unwrap()),
                        _ => None,
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }

    pub fn from_bencode(bencode_decoder: &mut Decoder) -> Result<Torrent, Error> {
        let mut torrent_result = Torrent {
            announce: None,
            piece_length_in_bytes: None,
            number_of_pieces: None,
            total_length_in_bytes: None,
            name: None,
            info_hash: None,
        };

        let bencode_object = bencode_decoder
            .next_object()
            .map_err(|_| Error::FailedToParseTorrentFile)?;

        match bencode_object {
            Some(Object::Dict(mut dict_decoder)) => {
                torrent_result.decode_dict(&mut dict_decoder)?;
            }
            None => (), // EOF
            _ => (),
        };

        torrent_result.number_of_pieces = Some(div_ceil(
            match torrent_result.total_length_in_bytes() {
                Some(length) => length,
                None => return Err(Error::TotalPiecesLengthNotFoundDuringParsing),
            },
            match torrent_result.piece_length_in_bytes() {
                Some(piece_length) => piece_length,
                None => return Err(Error::SinglePieceLengthNotFoundDuringParsing),
            },
        ));

        Ok(torrent_result)
    }
}

fn div_ceil(a: u32, b: u32) -> u32 {
    a / b + if a % b == 0 { 0 } else { 1 }
}
