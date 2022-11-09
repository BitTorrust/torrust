// TODO rename file name
use crate::http::Peer;
use crate::pwp::IntoBytes;
use crate::Error::FailedToConnectToPeer;
use std::io::prelude::*;
use std::net::TcpStream;

struct TCPSession {
    peer: Peer,
    steam: TcpStream,
}

impl TCPSession {
    pub fn new(self, peer: Peer) -> Result<TCPSession, Error> {
        let stream = TcpStream::connect(peer.socket_address()).map_err(|_| Error::FailedToConnectToPeer)?;
        Self {
            peer,
            steam: stream,
        }
    }

    /// Returns the number of bytes sent
    pub fn send(bittorrent_message: impl into_bytes) -> usize { 
        self.stream.write(bittorrent_message.into_bytes())
    }

    // Returns the number of bytes received
    pub fn receive() -> usize {
        self.stream.read()
    }
}


