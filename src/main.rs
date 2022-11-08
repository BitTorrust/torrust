mod torrent;
pub use torrent::Torrent;
mod error;
pub use error::Error;

mod app;
mod cli; // why is it necessary to declare the mod here and not in app
mod cli;
mod http;
mod pwp; // why is it necessary to declare the mod here and not in app

#[cfg(test)]
mod tests;

fn main() -> Result<(), Error> {
    let response = app::run();

    // Main program
    println!("{:?}", response);
    Ok(())
}
