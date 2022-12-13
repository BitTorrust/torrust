#[cfg(test)]
pub mod tests {
    use crate::state_machine::identity::{generate_random_identity, CLIENT_VERSION_ID};

    #[test]
    pub fn generate_peer_id() {
        let actual_peer_id = generate_random_identity();
        let mut expected_eighth_first_char = CLIENT_VERSION_ID.chars();

        for index in 0..8 {
            assert_eq!(
                actual_peer_id[index],
                match expected_eighth_first_char.next() {
                    Some(char) => char as u8,
                    None => {
                        panic!()
                    }
                }
            )
        }
    }
}
