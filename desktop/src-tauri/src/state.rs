use std::sync::RwLock;
use parser::log::Log;

pub struct AppState {
    pub logs: RwLock<Option<Vec<Log>>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            logs: RwLock::new(None),
        }
    }
}
