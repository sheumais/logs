use parser::log::Log;
use std::sync::RwLock;
use tauri_plugin_dialog::FilePath;

#[allow(dead_code)]
pub struct AppState {
    pub logs: RwLock<Option<Vec<Log>>>,
    pub log_files: RwLock<Option<Vec<FilePath>>>,
    pub live_log_folder: RwLock<Option<FilePath>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            logs: RwLock::new(None),
            log_files: RwLock::new(None),
            live_log_folder: RwLock::new(None),
        }
    }
}
