pub enum PeerToWireState {
    NotInterestedAndChoked,
    InterestedAndChoked,
    InterestedAndUnchoked,
    NotInterestedAndUnchoked,
}
pub struct PeerToWireCommunication {
    state: PeerToWireState,
}

impl PeerToWireCommunication {
    pub fn new() -> Self {
        PeerToWireCommunication {
            state: PeerToWireState::NotInterestedAndChoked,
        }
    }
    pub fn not_interested_and_choked() {
        println!(" I am in the Not interested and choked state");
    }

    pub fn interested_and_choked() {
        println!(" I am in the interested and choked state");
    }

    pub fn interested_and_unchoked() {
        println!(" I am in the interested and unchoked state");
    }

    pub fn not_interested_and_unchoked() {
        println!(" I am in the not interested and unchoked state");
    }

    pub fn state_transition(&self) {
        match &self.state {
            NotInterestedAndChoked => Self::not_interested_and_choked(),
            InterestedAndChoked => Self::interested_and_choked(),
            InterestedAndUnchoked => Self::interested_and_unchoked(),
            NotInterestedAndUnchoked => Self::not_interested_and_unchoked(),
        }
    }
}
