use parser::log::Log;
use std::sync::RwLock;
use tauri_plugin_dialog::FilePath;

pub struct AppState {
    pub logs: RwLock<Option<Vec<Log>>>,
    pub log_files: RwLock<Option<Vec<FilePath>>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            logs: RwLock::new(None),
            log_files: RwLock::new(None),
        }
    }
}
