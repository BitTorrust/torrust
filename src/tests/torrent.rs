#[cfg(test)]
pub mod test {
    use crate::{Error, Torrent};
    use bendy::decoding::Decoder;
    use std::{fs::File, io::Read, path::Path};

    #[test]
    pub fn parse_iceberg_image() -> Result<(), Error> {
        let filepath = Path::new("samples/iceberg.jpg.torrent");
        let mut file = File::open(filepath).map_err(|_| Error::FailedToOpenTorrentFile)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|_e| Error::FailedToReadTorrentFile)?;
        let mut bencode_decoder = Decoder::new(&buffer);
        let maybe_torrent = Torrent::from_bencode(&mut bencode_decoder);

        match maybe_torrent {
            Ok(torrent) => {
                assert_eq!(torrent.name().unwrap(), "iceberg.jpg");
                assert_eq!(
                    torrent.announce().unwrap(),
                    "http://127.0.0.1:6969/announce"
                );
                assert_eq!(torrent.number_of_pieces().unwrap(), 11);
                assert_eq!(
                    torrent.info_hash().unwrap(),
                    [
                        0x06, 0x71, 0x33, 0xac, 0xe5, 0xdd, 0x0c, 0x50, 0x27, 0xb9, 0x9d, 0xe5,
                        0xd4, 0xba, 0x51, 0x28, 0x28, 0x20, 0x8d, 0x5b
                    ]
                );
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
