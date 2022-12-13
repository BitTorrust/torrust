use {
    crate::torrent::Torrent,
    std::{fs, path::PathBuf},
};

pub fn is_file_on_disk_already(torrent: &Torrent, working_directory: &PathBuf) -> bool {
    let path = working_directory.join(torrent.name());

    match fs::metadata(path) {
        Ok(metadata) => metadata.len() == torrent.total_length_in_bytes().into(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use {
        crate::Torrent,
        std::path::{Path, PathBuf},
    };

    #[test]
    fn existing_file() {
        let path = Path::new("samples/upload/venon.jpg.torrent");
        let torrent = Torrent::from_file(path).unwrap();
        let working_directory = &PathBuf::from("samples/upload/");

        assert_eq!(true, is_file_on_disk_already(&torrent, working_directory));
    }

    #[test]
    fn non_existing_file() {
        let path = Path::new("samples/upload/venon.jpg.torrent");
        let torrent = Torrent::from_file(path).unwrap();
        let working_directory = &PathBuf::from("blabla/blabla/");

        assert_eq!(false, is_file_on_disk_already(&torrent, working_directory));
    }
}
