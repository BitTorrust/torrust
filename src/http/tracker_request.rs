use {crate::http::Event, crate::Error, reqwest::Url};

pub struct TrackerRequest {
    info_hash: [u8; 20],
    peer_id: [u8; 20],
    port: u16,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    compact: bool,
    event: Option<Event>,
}

impl TrackerRequest {
    pub fn new(
        info_hash: [u8; 20],
        peer_id: [u8; 20],
        port: u16,
        uploaded: usize,
        downloaded: usize,
        left: usize,
        compact: bool,
        event: Option<Event>,
    ) -> Self {
        Self {
            info_hash,
            peer_id,
            port,
            uploaded,
            downloaded,
            left,
            compact,
            event,
        }
    }

    pub fn into_url(self, host: &str, port: u16) -> Result<Url, Error> {
        let mut url = Url::parse(&format!(
            "http://{}:{}/announce?info_hash={}&peer_id={}",
            host,
            port,
            urlencoding::encode_binary(&self.info_hash),
            urlencoding::encode_binary(&self.peer_id),
        ))
        .map_err(|_| Error::FailedToParseUrl)?;

        self.append_parameters(&mut url);

        Ok(url)
    }

    fn append_parameters(&self, url: &mut Url) {
        url.query_pairs_mut()
            .append_pair("port", &format!("{}", self.port))
            .append_pair("uploaded", &format!("{}", self.uploaded))
            .append_pair("downloaded", &format!("{}", self.downloaded))
            .append_pair("left", &format!("{}", self.left))
            .append_pair("compact", if self.compact { "1" } else { "0" });

        if let Some(event) = &self.event {
            url.query_pairs_mut()
                .append_pair("event", &event.to_string());
        }
    }
}
