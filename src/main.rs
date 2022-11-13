mod torrent;
pub use torrent::Torrent;

mod error;
pub use error::Error;

mod file_management;
pub use file_management::BlockReaderWriter;

mod app;
use app::App;

mod cli;
mod http;
mod pwp;
mod tcp;

mod pwp_communication;
pub use pwp_communication::PeerToWireCommunication;

#[cfg(test)]
mod tests;

fn main() -> Result<(), Error> {
    //App::run()
    let p2w_communication = PeerToWireCommunication::new();
    p2w_communication.state_transition();
    Ok(())
}
