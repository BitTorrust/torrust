#[cfg(test)]
pub mod torrent {
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
                println!("Print torrent struct {:?}", torrent);

                assert_eq!(torrent.name().unwrap(), "iceberg.jpg");
                assert_eq!(
                    torrent.announce().unwrap(),
                    "http://127.0.0.1:6969/announce"
                );
                assert_eq!(torrent.number_of_pieces().unwrap(), 11);
                //assert_eq!(String::from_utf8(*torrent.info_hash().unwrap()).unwrap().make_ascii_lowercase(), "067133ace5dd0c5027b99de5d4ba512828208d5b");
                // TODO
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
