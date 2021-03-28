use std::process::Child;
use std::sync::Mutex;

/// A simple in-memory DB to store logging state
pub type Db = Mutex<LoggerState>;
#[derive(Debug)]
pub struct LoggerState {
    pub id: u64,
    pub path: Option<String>,
    pub previous_path: Option<String>,
    pub call_count: u32,
    pub active: bool,
    pub command: Option<Child>,
}

impl LoggerState {
    pub fn new() -> LoggerState {
        LoggerState {
            id: 0,
            path: None,
            previous_path: None,
            command: None,
            call_count: 0,
            active: false,
        }
    }
}
