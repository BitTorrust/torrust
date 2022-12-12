use rand::prelude::*;

pub const CLIENT_VERSION_ID: &str = "-Tr0001-";

// Generate a random peer id (using azureus-style : "-AB1234-...")
pub fn generate_random_identity() -> [u8; 20] {
    let mut version_information = CLIENT_VERSION_ID.chars();
    let mut id = [0 as u8; 20];

    let mut random_number_generator = rand::thread_rng();
    for index in 0..20 {
        id[index] = match version_information.next() {
            Some(char) => char as u8,
            None => random_number_generator.gen(),
        };
    }

    id
}
