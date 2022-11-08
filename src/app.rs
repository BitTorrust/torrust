/* Things are always a struct until they become something else */

use crate::cli::Args;
use crate::error::Error;
use crate::http::{Event, TrackerRequest, TrackerResponse};
use crate::torrent::Torrent;
use bendy::decoding::Decoder;
use clap::Parser;
use reqwest::Url;
use std::{convert::TryInto, fs, path::PathBuf};

mod tracker;
pub use tracker::Tracker;

static PEER_ID: [u8; 20] = [
    0xDE, 0xAD, 0xBE, 0xEF, 0xBA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
    0xAA, 0xAA, 0xAA, 0xAD,
];

pub fn parse_torrent(torrent_file_path: &PathBuf) -> Torrent {
    let torrent_file_content = fs::read(torrent_file_path).unwrap();
    let mut bencode_decoder = Decoder::new(&torrent_file_content);
    let torrent = Torrent::from_bencode(&mut bencode_decoder).unwrap();

    return torrent;
}

pub fn get_tracker(torrent: &Torrent) -> Tracker {
    let tracker_url = Url::parse(torrent.announce().unwrap()).unwrap();
    let host = tracker_url.host().unwrap().to_string();
    let port = tracker_url.port().unwrap();
    let tracker = Tracker::new(host, port);

    return tracker;
}

pub fn build_tracker_request(torrent: &Torrent) -> TrackerRequest {
    let info_hash_vector = torrent.info_hash().unwrap();
    let info_hash: [u8; 20] = info_hash_vector.try_into().unwrap();
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

    return tracker_request;
}

pub fn send_request(
    tracker_request: TrackerRequest,
    tracker: Tracker,
) -> Result<TrackerResponse, Error> {
    let tracker_host = tracker.get_tracker_host();
    let tracker_port = tracker.get_tracker_port();
    let url = tracker_request
        .into_url(tracker_host, tracker_port)
        .unwrap();

    let mut response = reqwest::blocking::get(url).unwrap();
    let mut bencode = Vec::new();
    response.copy_to(&mut bencode).unwrap();
    let parsed_response = TrackerResponse::from_bencode(&bencode);
    return parsed_response;
}

pub fn run() -> Result<TrackerResponse, Error> {
    let args = Args::parse();
    let torrent = parse_torrent(args.torrent_file());
    let tracker_request = build_tracker_request(&torrent);
    let tracker = get_tracker(&torrent);
    let response = send_request(tracker_request, tracker);

    return response;
}
