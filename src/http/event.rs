use std::fmt;

#[derive(Debug)]
pub enum Event {
    Started,
    Stopped,
    Completed,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Event::Started => "started",
            Event::Stopped => "stopped",
            Event::Completed => "completed",
        };

        write!(f, "{}", text)
    }
}
