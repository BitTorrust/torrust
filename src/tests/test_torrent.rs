#[cfg(test)]
mod tests {
    use crate::Error;
    use crate::Torrent;
    use bendy::decoding::Decoder;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn parse_iceberg_image() -> Result<(), Error> {
        let filepath = "samples/iceberg.jpg.torrent";
        let mut file = File::open(filepath).map_err(|_e| Error::FailedToOpenTorrentFile)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|_e| Error::FailedToReadTorrentFile)?;
        let mut bencode_object = Decoder::new(&buffer);
        let result = Torrent::from_bencode(&mut bencode_object);

        return match result {
            Ok(torrent) => {
                assert!(true);
                println!("Print torrent struct {:?}", torrent);

                assert_eq!(torrent.name(), "iceberg.jpg");
                assert_eq!(torrent.announce(), "http://127.0.0.1:6969/announce");
                assert_eq!(torrent.number_of_pieces(), 11);
                Ok(())
            }
            Err(e) => panic!("{:?}", e),
        };
    }
}
