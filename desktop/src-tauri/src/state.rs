use parser::log::Log;
use tauri_plugin_dialog::FilePath;
use std::sync::RwLock;

pub struct AppState {
    pub logs: RwLock<Option<Vec<Log>>>,
    pub log_file: RwLock<Option<FilePath>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            logs: RwLock::new(None),
            log_file: RwLock::new(None),
        }
    }
}
