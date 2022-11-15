use {
    crate::{
        http::Peer,
        pwp::{IntoBytes, Message},
        Error,
    },
    std::{
        io::{self, prelude::*},
        net::TcpStream,
    },
};

/// This is a temporary struct that changes the method receive from
///
/// TCPSession::receive(&self, buffer: &mut [u8]) -> Result<usize, io::Error>
///
/// to
///
/// TCPSession::receive(&mut self) -> Result<Option<Message>, io::Error>
///
/// It will be used to develop the other parts of the code while
/// the changes to the real TCPSession structure are not finished.
#[derive(Debug)]
pub struct TCPSessionMock {
    peer: Peer,
    steam: TcpStream,
}

impl TCPSessionMock {
    pub fn connect(peer: Peer) -> Result<Self, Error> {
        let stream =
            TcpStream::connect(peer.socket_address()).map_err(|_| Error::FailedToConnectToPeer)?;
        Ok(Self {
            peer,
            steam: stream,
        })
    }

    fn steam(&self) -> &TcpStream {
        &self.steam
    }

    /// Returns the number of bytes sent
    pub fn send(&self, bittorrent_message: impl IntoBytes) -> Result<usize, io::Error> {
        self.steam().write(&(bittorrent_message.into_bytes()))
    }

    pub fn send_bytes(&self, bytes: Vec<u8>) -> Result<usize, io::Error> {
        self.steam.write(&bytes)
    }

    /// Write the received bytes in the buffer
    /// Returns the number of bytes received
    pub fn receive(&mut self) -> Result<Option<Message>, io::Error> {
        unimplemented!()
    }
}
