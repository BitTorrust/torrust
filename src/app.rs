/* Things are always a struct until they become something else */

use crate::{
    cli::Args,
    error::Error,
    http::{Event, TrackerRequest, TrackerResponse},
    torrent::Torrent,
};
use bendy::decoding::Decoder;
use clap::Parser;
use reqwest::Url;
use std::{fs, path::PathBuf};

mod tracker;

use self::tracker::TrackerAddress;

static PEER_ID: [u8; 20] = [
    0xDE, 0xAD, 0xBE, 0xEF, 0xBA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
    0xAA, 0xAA, 0xAA, 0xAD,
];

pub struct App {}

impl App {
    fn parse_torrent(torrent_file_path: &PathBuf) -> Torrent {
        let torrent_file_content = fs::read(torrent_file_path).unwrap();
        let mut bencode_decoder = Decoder::new(&torrent_file_content);
        let torrent = Torrent::from_bencode(&mut bencode_decoder).unwrap();

        torrent
    }

    fn tracker_address(torrent: &Torrent) -> Result<TrackerAddress, Error> {
        let tracker_url =
            Url::parse(torrent.announce().unwrap()).map_err(|_| Error::InvalidURLAddress)?;

        let tracker_address = TrackerAddress::from_url(tracker_url);

        Ok(tracker_address)
    }

    fn build_tracker_request(torrent: &Torrent) -> TrackerRequest {
        let info_hash = torrent.info_hash().unwrap();
        let left_to_download = torrent.total_length_in_bytes().unwrap();
        let event = Event::Started;
        let tracker_request = TrackerRequest::new(
            info_hash,
            PEER_ID,
            6882,
            0,
            0,
            left_to_download as usize,
            true,
            Some(event),
        );

        tracker_request
    }

    fn send_request(
        tracker_request: TrackerRequest,
        tracker: TrackerAddress,
    ) -> Result<TrackerResponse, Error> {
        let url = tracker_request
            .into_url(tracker.host(), tracker.port())
            .unwrap();

        let mut response = reqwest::blocking::get(url).unwrap();
        let mut bencode = Vec::new();
        response.copy_to(&mut bencode).unwrap();
        let parsed_response = TrackerResponse::from_bencode(&bencode);

        parsed_response
    }

    pub fn run() -> Result<(), Error> {
        let args = Args::parse();
        let torrent = Self::parse_torrent(args.torrent_file());
        let tracker_request = Self::build_tracker_request(&torrent);
        let tracker_address = Self::tracker_address(&torrent)?;
        let response = Self::send_request(tracker_request, tracker_address)?;

        println!("{:?}", response);
        Ok(())
    }
}
