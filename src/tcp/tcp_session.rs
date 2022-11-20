use crate::http::Peer;
use crate::pwp::IntoBytes;
use crate::Error;
use std::io::{self, prelude::*, BufReader};
use std::net::{SocketAddr, SocketAddrV4, TcpListener, TcpStream};

pub struct TCPSession {
    peer: Peer,
    steam: TcpStream,
}

impl TCPSession {
    pub fn connect(peer: Peer) -> Result<TCPSession, Error> {
        let stream =
            TcpStream::connect(peer.socket_address()).map_err(|_| Error::FailedToConnectToPeer)?;
        Ok(Self {
            peer,
            steam: stream,
        })
    }

    pub fn accept(listener: TcpListener) -> Result<TCPSession, Error> {
        let (stream, socket_address) = listener
            .accept()
            .map_err(|_| Error::FailedToConnectToPeer)?;

        let peer = Peer::from_socket_address(socket_address);

        Ok(Self {
            peer,
            steam: stream,
        })
    }

    pub fn listen() -> Result<TcpListener, Error> {
        let listener =
            TcpListener::bind("127.0.0.1:6882").map_err(|_| Error::FailedToCreateTcpListener)?;
        Ok(listener)
    }

    fn steam(&self) -> &TcpStream {
        &self.steam
    }

    /// Returns the number of bytes sent
    pub fn send(&self, bittorrent_message: impl IntoBytes) -> Result<usize, io::Error> {
        self.steam().write(&(bittorrent_message.into_bytes()))
    }

    /// Write the received bytes in the buffer
    /// Returns the number of bytes received
    pub fn receive(&self, buffer: &mut [u8]) -> io::Result<usize> {
        let mut response = BufReader::new(self.steam());
        response.read(buffer)

        //self.steam().read(buffer)
    }
}
