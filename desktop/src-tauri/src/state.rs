use dirs::data_local_dir;
use esologtool_common::UpdateInformation;
use reqwest::Client;
use std::{env::temp_dir, fs::{self, create_dir_all, File}, io::Read, path::PathBuf, sync::{atomic::AtomicBool, Arc, RwLock}, time::Duration};
use tauri_plugin_dialog::FilePath;
use cookie_store::CookieStore;
use reqwest_cookie_store::CookieStoreMutex;

pub fn cookie_file_path() -> PathBuf {
    let mut dir = data_local_dir().unwrap_or_else(temp_dir);
    dir.push("eso-log-tool");
    create_dir_all(&dir).ok();
    dir.push("cookies.json");
    dir
}

pub fn cookie_folder_path() -> PathBuf {
    let mut dir = data_local_dir().unwrap_or_else(temp_dir);
    dir.push("eso-log-tool");
    create_dir_all(&dir).ok();
    dir
}

fn build_http_client_with_store(store: Arc<CookieStoreMutex>) -> Client {
    Client::builder()
        .cookie_provider(store.clone())
        .timeout(Duration::from_secs(10))
        .user_agent("eso-log-tool")
        .build()
        .expect("valid client")
}

fn load_cookie_store() -> Arc<CookieStoreMutex> {
    let path = cookie_file_path();
    log::trace!("Loading cookies from: {path:?}");
    if let Ok(mut file) = File::open(&path) {
        let mut data = String::new();
        if file.read_to_string(&mut data).is_ok() {
            if let Ok(store) = serde_json::from_str::<CookieStore>(&data) {
                log::trace!("Loaded cookies: {} cookies", store.iter_any().count());
                return Arc::new(CookieStoreMutex::new(store));
            } else {
                log::error!("Failed to deserialize cookies");
            }
        } else {
            log::error!("Failed to read cookie file");
        }
    } else {
        log::warn!("No cookie file found at {path:?}");
    }
    Arc::new(CookieStoreMutex::default())
}

fn save_cookie_store(store: &CookieStoreMutex) {
    let path = cookie_file_path();
    if let Ok(store) = store.lock() {
        let count = store.iter_any().count();
        log::trace!("Saving {count} cookies to {path:?}");
        if let Ok(json) = serde_json::to_string(&*store) {
            if let Err(e) = fs::write(&path, json) {
                log::error!("Failed to write cookie file: {e}");
            }
        } else {
            log::error!("Failed to serialize cookies");
        }
    } else {
        log::warn!("Failed to lock cookie store for saving");
    }
}

pub struct HttpState {
    pub client: Client,
    pub cookie_store: Arc<CookieStoreMutex>,
}

impl HttpState {
    pub fn new() -> Self {
        let cookie_store = load_cookie_store();
        let client = build_http_client_with_store(cookie_store.clone());
        Self { client, cookie_store }
    }

    pub fn save_cookies(&self) {
        save_cookie_store(&self.cookie_store);
    }
}

pub struct AppState {
    pub log_files: RwLock<Option<Vec<FilePath>>>,
    pub live_log_folder: RwLock<Option<FilePath>>,
    pub http: RwLock<HttpState>,
    pub esolog_code: RwLock<Option<String>>,
    pub upload_cancel_flag: Arc<AtomicBool>,
    pub update: RwLock<Option<UpdateInformation>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            log_files: RwLock::new(None),
            live_log_folder: RwLock::new(None),
            http: RwLock::new(HttpState::new()),
            esolog_code: RwLock::new(None),
            upload_cancel_flag: Arc::new(AtomicBool::new(false)),
            update: RwLock::new(None),
        }
    }
}
