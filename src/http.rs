mod tracker_request;
mod tracker_response;

pub use tracker_request::{Event, TrackerRequest};
pub use tracker_response::{Peer, TrackerResponse};

#[cfg(test)]
mod test {
    use crate::http::{Event, TrackerRequest, TrackerResponse};

    static INFO_ID: [u8; 20] = [
        0x06, 0x71, 0x33, 0xAC, 0xE5, 0xDD, 0x0C, 0x50, 0x27, 0xB9, 0x9D, 0xE5, 0xD4, 0xBA, 0x51,
        0x28, 0x28, 0x20, 0x8D, 0x5B,
    ];

    static PEER_ID: [u8; 20] = [
        0xDE, 0xAD, 0xBE, 0xEF, 0xBA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
        0xAA, 0xAA, 0xAA, 0xAA, 0xAD,
    ];

    static TRACKER_HOSTNAME: &'static str = "127.0.0.1";
    static TRACKER_PORT: u16 = 6969;

    #[test]
    fn request_into_url() {
        let url_builder = |event| {
            TrackerRequest::new(INFO_ID, PEER_ID, 6882, 0, 0, 356639, true, Some(event))
                .into_url(TRACKER_HOSTNAME, TRACKER_PORT)
                .unwrap()
        };

        let url = url_builder(Event::Started);
        assert_eq!(url.as_str(), "http://127.0.0.1:6969/announce?\
                                    info_hash=%06q3%AC%E5%DD%0CP%27%B9%9D%E5%D4%BAQ%28%28%20%8D%5B\
                                    &peer_id=%DE%AD%BE%EF%BA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AD\
                                    &port=6882&uploaded=0&downloaded=0&left=356639&compact=1&event=started");

        let url = url_builder(Event::Stopped);
        assert_eq!(url.as_str(), "http://127.0.0.1:6969/announce?\
                                    info_hash=%06q3%AC%E5%DD%0CP%27%B9%9D%E5%D4%BAQ%28%28%20%8D%5B\
                                    &peer_id=%DE%AD%BE%EF%BA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AD\
                                    &port=6882&uploaded=0&downloaded=0&left=356639&compact=1&event=stopped");

        let url = url_builder(Event::Completed);
        assert_eq!(url.as_str(), "http://127.0.0.1:6969/announce?\
                                    info_hash=%06q3%AC%E5%DD%0CP%27%B9%9D%E5%D4%BAQ%28%28%20%8D%5B\
                                    &peer_id=%DE%AD%BE%EF%BA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AD\
                                    &port=6882&uploaded=0&downloaded=0&left=356639&compact=1&event=completed");
    }

    // This test is ignored because it requires a tracker running on 127.0.0.1:6969.
    // It actually does not test anything, since the response has to be analysed
    // manually and also depends on the file being served. I'm only keeping this
    // test here until we finish the basic integration because it helps to manually
    // inspect what the tracker is responding us.
    #[test]
    #[ignore]
    fn tracker_http_request() {
        let event = Event::Started;
        let request = TrackerRequest::new(INFO_ID, PEER_ID, 6882, 0, 0, 356639, true, Some(event));
        let url = request.into_url(TRACKER_HOSTNAME, TRACKER_PORT).unwrap();

        let mut response = reqwest::blocking::get(url).unwrap();
        let mut bencode = Vec::new();
        response.copy_to(&mut bencode).unwrap();

        let parsed_response = TrackerResponse::from_bencode(&bencode);
        println!("{:?}", parsed_response);
    }
}
