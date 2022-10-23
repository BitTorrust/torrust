mod tracker_request;
mod tracker_response;

pub use tracker_request::{Event, TrackerRequest};
pub use tracker_response::{Peer, TrackerResponse};

#[cfg(test)]
mod test {
    use crate::http::{Event, TrackerRequest};

    #[test]
    fn request_into_url() {
        let info_id = [
            0x06, 0x71, 0x33, 0xAC, 0xE5, 0xDD, 0x0C, 0x50, 0x27, 0xB9, 0x9D, 0xE5, 0xD4, 0xBA,
            0x51, 0x28, 0x28, 0x20, 0x8D, 0x5B,
        ];

        let peer_id = [
            0xDE, 0xAD, 0xBE, 0xEF, 0xBA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
            0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAD,
        ];

        let request_builder =
            |event| TrackerRequest::new(info_id, peer_id, 6882, 0, 0, 356639, true, Some(event));

        let hostname = "127.0.0.1";
        let port = 6969;

        let url = request_builder(Event::Started).into_url(hostname, port);
        assert_eq!(url.as_str(), "http://127.0.0.1:6969/announce?info_hash=%06q3%AC%E5%DD%0CP%27%B9%9D%E5%D4%BAQ%28%28%20%8D%5B&peer_id=%DE%AD%BE%EF%BA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AD&port=6882&uploaded=0&downloaded=0&left=356639&compact=1&event=started");

        let url = request_builder(Event::Stopped).into_url(hostname, port);
        assert_eq!(url.as_str(), "http://127.0.0.1:6969/announce?info_hash=%06q3%AC%E5%DD%0CP%27%B9%9D%E5%D4%BAQ%28%28%20%8D%5B&peer_id=%DE%AD%BE%EF%BA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AD&port=6882&uploaded=0&downloaded=0&left=356639&compact=1&event=stopped");

        let url = request_builder(Event::Completed).into_url(hostname, port);
        assert_eq!(url.as_str(), "http://127.0.0.1:6969/announce?info_hash=%06q3%AC%E5%DD%0CP%27%B9%9D%E5%D4%BAQ%28%28%20%8D%5B&peer_id=%DE%AD%BE%EF%BA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AA%AD&port=6882&uploaded=0&downloaded=0&left=356639&compact=1&event=completed");
    }
}
