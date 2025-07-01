use parser::log::Log;
use reqwest::{cookie::Jar, Client};
use std::{sync::{Arc, RwLock}, time::Duration};
use tauri_plugin_dialog::FilePath;

fn build_http_client() -> (Client, Arc<Jar>) {
    let jar = Arc::new(Jar::default());
    let client = Client::builder()
        .cookie_provider(jar.clone())
        .timeout(Duration::from_secs(10))
        .user_agent("eso-log-tool")
        .build()
        .expect("valid client");
    (client, jar)
}

pub struct HttpState {
    pub client: Client,
    jar: Arc<Jar>,
}

impl HttpState {
    fn new() -> Self {
        let (client, jar) = build_http_client();
        Self { client, jar }
    }
}

#[allow(dead_code)]
pub struct AppState {
    pub logs: RwLock<Option<Vec<Log>>>,
    pub log_files: RwLock<Option<Vec<FilePath>>>,
    pub live_log_folder: RwLock<Option<FilePath>>,
    pub http: RwLock<HttpState>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            logs: RwLock::new(None),
            log_files: RwLock::new(None),
            live_log_folder: RwLock::new(None),
            http: RwLock::new(HttpState::new()),
        }
    }
}
