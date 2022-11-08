mod torrent;
pub use torrent::Torrent;
mod error;
pub use error::Error;

mod app;
mod cli;
mod http;
mod pwp;

#[cfg(test)]
mod tests;

fn main() -> Result<(), Error> {
    let response = app::run();

    // Main program
    println!("{:?}", response);
    Ok(())
}
