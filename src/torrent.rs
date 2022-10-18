use std::str::FromStr;
use bendy::decoding::{Object, Decoder};
use crate::Error;

#[derive(Debug)]
pub struct Torrent {
    announce: String,
    piece_length_in_bytes: u32,
    number_of_pieces: u32,
    total_length_in_bytes: u32,
    name: String,
}

impl Torrent {
    pub fn announce(&self) -> &str {
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

    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn from_bencode(bencode_decoder: &mut Decoder) -> Result<Torrent, Error> {
        let mut announce = None;
        let mut piece_length_in_bytes = None;
        let mut total_length_in_bytes = None;
        let mut name = None;

        let bencode_object = bencode_decoder.next_object().map_err(|_e| Error::FailedToParserTorrentFile)?;
        match bencode_object {
            None => (), // EOF
            Some(Object::Dict(mut dict)) => {
                while let Ok(Some(pair)) = dict.next_pair() {
                    let key = String::from_utf8(pair.0.to_vec()).unwrap();

                    // DEBUG
                    println!("key: {:?}, type: {}", key, match pair.1 {
                        Object::Bytes(_) => "Bytes",
                        Object::List(_) => "List",
                        Object::Dict(_) => "Dict",
                        Object::Integer(_) => "Integer",                        
                    });


                    match key.as_str() {
                        "announce" => announce = match pair.1 {
                            Object::Bytes(byte) => Some(String::from_utf8(byte.to_vec()).unwrap()),
                            _ => None
                        },
                        "info" => match pair.1 {
                            Object::Dict(mut info_dict) => {
                                while let Ok(Some(info_pair)) = info_dict.next_pair() {
                                    let key = String::from_utf8(info_pair.0.to_vec()).unwrap();
                                    
                                    println!("info key: {:?}, type: {}", key, match info_pair.1 {
                                        Object::Bytes(_) => "Bytes",
                                        Object::List(_) => "List",
                                        Object::Dict(_) => "Dict",
                                        Object::Integer(_) => "Integer",                        
                                    });

                                    match key.as_str() {
                                        "piece length" => piece_length_in_bytes = match info_pair.1 {
                                            Object::Integer(string) => Some(u32::from_str(string).unwrap()),
                                            _ => None
                                        },
                                        "length" => total_length_in_bytes =  match info_pair.1 {
                                            Object::Integer(string) => Some(u32::from_str(string).unwrap()),
                                            _ => None
                                        },
                                        "name" => name = match info_pair.1 {
                                            Object::Bytes(byte) => Some(String::from_utf8(byte.to_vec()).unwrap()),
                                            _ => None
                                        },
                                        _ => (),
                                    };
                                }
                            },
                            _ => (),
                        },
                        _ => (),
                    };
                }
            },
            _ => (),
        };
        Ok(Torrent {
            announce: announce.expect("failed to parse announce field from the torrent file"),
            piece_length_in_bytes: piece_length_in_bytes.expect("failed to parse announce field from the torrent file"),
            number_of_pieces: div_ceil(total_length_in_bytes.expect("failed to parse announce field from the torrent file"),
                piece_length_in_bytes.expect("failed to parse announce field from the torrent file")),
            total_length_in_bytes: total_length_in_bytes.expect("failed to parse announce field from the torrent file"),
            name: name.expect("failed to parse announce field from the torrent file"),

        })
    }
}

fn div_ceil(a: u32, b: u32) -> u32 {
    a / b + if a % b == 0 { 0 } else { 1 }
}
