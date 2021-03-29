use std::sync::Mutex;
use std::{collections::HashMap, process::Child};

/// A simple in-memory DB to store logging state
pub type Db = Mutex<LoggerState>;
#[derive(Debug)]
pub struct LoggerState {
    pub serial_number: u64,
    pub call_count: u32,
    pub active: bool,
    pub components: HashMap<String, ComponentState>,
}

#[derive(Debug)]
pub struct ComponentState {
    pub name: String,
    pub log_path: Option<String>,
    pub previous_log_path: Option<String>,
    pub active: bool,
    pub command: Option<Child>,
}

impl LoggerState {
    pub fn new() -> LoggerState {
        let components = HashMap::new();
        components["sc_phy"] = ComponentState::new("sc_phy".to_string());
        components["sc_scheduler"] = ComponentState::new("sc_scheduler".to_string());
        components["sc_sm"] = ComponentState::new("sc_sm".to_string());
        components["kernel"] = ComponentState::new("kernel".to_string());
        LoggerState {
            serial_number: 0,
            call_count: 0,
            active: false,
            components: components,
        }
    }
}

impl ComponentState {
    pub fn new(name: String) -> ComponentState {
        ComponentState {
            name: name,
            log_path: None,
            previous_log_path: None,
            command: None,
            active: false,
        }
    }
}
