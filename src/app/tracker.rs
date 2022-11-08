pub struct Tracker {
    host : String,
    port: u16,   
}

impl Tracker {
    pub fn new(
        host: String,
        port: u16,
    ) -> Self {
        Self {
            host,
            port,
        }
    }

    pub fn get_tracker_host(&self) -> &String {
        &self.host
    }

    pub fn get_tracker_port(&self) -> u16 {
        self.port
    }
    pub fn set_tracker_host(&mut self,host:String) {
        self.host = host;
    }

    pub fn set_tracker_port(&mut self, port:u16) {
        self.port = port;
    }

}
