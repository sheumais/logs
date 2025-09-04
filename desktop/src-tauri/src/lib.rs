use cli::{esologs_convert::{build_master_table, build_report_segment, event_timestamp, split_and_zip_log_by_fight, write_zip_with_logtxt, ESOLogProcessor}, esologs_format::{EncounterReportCode, LoginResponse, UploadSettings, ESO_LOGS_COM_VERSION, ESO_LOGS_PARSER_VERSION, LINE_COUNT_FOR_PROGRESS}, log_edit::{handle_line, CustomLogData}};
use reqwest::{multipart::{Form, Part}, Client};
use serde_json::json;
use state::AppState;
use tauri_plugin_updater::UpdaterExt;
use std::{
    env::temp_dir, fs::{self, create_dir_all, File, OpenOptions}, io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}, sync::atomic::Ordering::SeqCst, thread, time::{Duration, SystemTime, UNIX_EPOCH}
};
use tauri::{async_runtime::spawn_blocking, path::BaseDirectory, Emitter, Manager, State, Window};
use tauri_plugin_dialog::{DialogExt, FilePath};
use ftail::Ftail;
use log::LevelFilter;

use crate::state::{cookie_file_path, cookie_folder_path};
mod state;

#[tauri::command]
fn modify_log_file(window: Window, state: State<'_, AppState>) -> Result<(), String> {
    let paths_guard = state.log_files.read().unwrap();
    let file_paths = paths_guard.as_ref().ok_or("No file paths set")?;
    let file_path = file_paths.get(0).ok_or("No file path in vector")?;
    let path_ref = file_path.as_path().ok_or("Invalid file path")?;
    let file = File::open(path_ref).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(&file);
    let total_lines = reader.lines().count();

    let parent = path_ref.parent().unwrap_or_else(|| Path::new("."));
    let orig_file_name = path_ref.file_stem().and_then(|s| s.to_str()).unwrap_or("Encounter");
    let new_file_name = format!("{}-MODIFIED.log", orig_file_name);
    let new_path = parent.join(new_file_name);
    let file = File::open(path_ref).map_err(|e| format!("Failed to reopen file: {}", e))?;
    let reader = BufReader::new(file);

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&new_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut writer = BufWriter::new(file);
    let mut custom_log_data = CustomLogData::new();

    let mut processed = 0;
    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                log::warn!("Error reading line: {}", e);
                continue;
            }
        };
        let modified_line = handle_line(line, &mut custom_log_data);
        for entry in modified_line {
            writeln!(writer, "{entry}").ok();
        }
        processed += 1;

        if processed % LINE_COUNT_FOR_PROGRESS == 0 || processed == total_lines {
            let progress = (processed * 100 / total_lines).min(100);
            window
                .emit("log_modify_progress", progress)
                .map_err(|e| format!("Failed to emit progress: {}", e))?;
        }
    }

    Ok(())
}

#[tauri::command]
fn split_encounter_file_into_log_files(window: Window, state: State<'_, AppState>) -> Result<(), String> {
    let paths_guard = state.log_files.read().map_err(|e| e.to_string())?;
    let file_paths = paths_guard.as_ref().ok_or("No file paths set")?;
    let file_path = file_paths.get(0).ok_or("No file path in vector")?;
    let path_ref = file_path.as_path().ok_or("Invalid file path")?;
    let file = File::open(path_ref).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(&file);
    let total_lines = reader.lines().count();
    let file = File::open(path_ref).map_err(|e| format!("Failed to reopen file: {}", e))?;
    let reader = BufReader::new(file);

    let mut current_writer: Option<BufWriter<File>> = None;

    let mut processed = 0;
    for line_result in reader.lines() {
        let line = line_result.map_err(|e| format!("Error reading line: {}", e))?;
        let mut parts = line.splitn(4, ',');
        let _ = parts.next();
        let linetype = parts.next();
        let timestamp = parts.next();

        if let (Some("BEGIN_LOG"), Some(time)) = (linetype, timestamp) {
            let out_name = format!("Split-Encounter-{}.log", time);
            let parent = file_path
                .as_path()
                .ok_or("Invalid file path")?
                .parent()
                .unwrap_or_else(|| Path::new("."));
            let out_path = parent.join(out_name);
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&out_path)
                .map_err(|e| format!("Failed to create output file: {}", e))?;
            current_writer = Some(BufWriter::new(file));
        }

        if let Some(writer) = current_writer.as_mut() {
            writeln!(writer, "{line}")
                .map_err(|e| format!("Failed to write to split file: {}", e))?;
        }

        processed += 1;
        if processed % LINE_COUNT_FOR_PROGRESS == 0 || processed == total_lines {
            let progress = (processed * 100 / total_lines).min(100);
            window
                .emit("log_split_progress", progress)
                .map_err(|e| format!("Failed to emit progress: {}", e))?;
        }
    }

    if let Some(mut writer) = current_writer {
        writer
            .flush()
            .map_err(|e| format!("Failed to flush writer: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
fn combine_encounter_log_files(window: Window, state: State<'_, AppState>) -> Result<(), String> {
    let paths_guard = state.log_files.read().map_err(|e| e.to_string())?;
    let file_paths = paths_guard.as_ref().ok_or("No file paths set")?;
    if file_paths.is_empty() {
        return Err("No files provided".into());
    }

    let first_file = File::open(&file_paths[0].as_path().unwrap())
        .map_err(|e| format!("Failed to open first file: {}", e))?;
    let mut first_reader = BufReader::new(first_file);
    let mut first_line = String::new();
    first_reader
        .read_line(&mut first_line)
        .map_err(|e| format!("Failed to read first line: {}", e))?;
    let start_timestamp = first_line
        .splitn(4, ',')
        .nth(2)
        .ok_or("Malformed BEGIN_LOG line in first file")?
        .trim();

    let last_path = file_paths
        .last()
        .unwrap()
        .as_path()
        .ok_or("Invalid file path")?;
    let last_file =
        File::open(last_path).map_err(|e| format!("Failed to open last file: {}", e))?;
    let mut last_reader = BufReader::new(last_file);
    let mut last_line = String::new();
    last_reader
        .read_line(&mut last_line)
        .map_err(|e| format!("Failed to read last line: {}", e))?;
    let end_timestamp = last_line
        .splitn(4, ',')
        .nth(2)
        .ok_or("Malformed BEGIN_LOG line in last file")?
        .trim();

    let out_name = format!(
        "Combined-Encounter-{}-{}.log",
        start_timestamp, end_timestamp
    );
    let parent = file_paths[0]
        .as_path()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| Path::new("."));
    let out_path = parent.join(out_name);
    let mut out_file =
        File::create(&out_path).map_err(|e| format!("Failed to create output file: {}", e))?;

    let mut total_lines = 0;
    for path in file_paths {
        let file = File::open(path.as_path().unwrap())
            .map_err(|e| format!("Failed to open file for counting: {}", e))?;
        let reader = BufReader::new(file);
        total_lines += reader.lines().count();
    }

    let mut processed = 0;
    for path in file_paths {
        let file = File::open(path.as_path().unwrap())
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            writeln!(out_file, "{line}").map_err(|e| format!("Failed to write line: {}", e))?;
            processed += 1;
            if processed % LINE_COUNT_FOR_PROGRESS == 0 || processed == total_lines {
                let progress = (processed * 100 / total_lines).min(100);
                window
                    .emit("log_combine_progress", progress)
                    .map_err(|e| format!("Failed to emit progress: {}", e))?;
            }
        }
    }

    Ok(())
}

#[tauri::command]
fn live_log_from_folder(window: Window, app_state: State<'_, AppState>) -> Result<(), String> {
    let folder_guard = app_state.live_log_folder.read().map_err(|e| e.to_string())?;
    let folder = folder_guard.as_ref().ok_or("No folder selected")?.clone();

    let folder_pathbuf = folder.as_path().unwrap().to_path_buf();
    let input_path = folder_pathbuf.join("Encounter.log");

    let mut output_folder_pathbuf = folder.as_path().unwrap().to_path_buf();
    output_folder_pathbuf.push("LogToolLive");
    std::fs::create_dir_all(&output_folder_pathbuf)
        .map_err(|e| format!("Failed to create output folder: {}", e))?;
    let output_path = output_folder_pathbuf.join("Encounter.log");

    let window = window.clone();
    thread::spawn(move || {
        let mut input_file = loop {
            match OpenOptions::new().read(true).open(&input_path) {
                Ok(f) => break f,
                Err(_) => {
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            }
        };
        let output_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&output_path)
            .expect("Failed to open output file");
        let mut writer = BufWriter::new(&output_file);
        let mut pos = 0u64;
        let mut buffer = Vec::new();

        loop {
            input_file
                .seek(SeekFrom::Start(pos))
                .expect("Failed to seek input file");

            buffer.clear();
            let mut reader = BufReader::new(&input_file);
            let bytes_read = reader.read_to_end(&mut buffer).expect("Failed to read input file");

            if bytes_read == 0 {
                thread::sleep(Duration::from_secs(5));
                continue;
            }

            if let Some(last_newline_offset) = buffer.iter().rposition(|&b| b == b'\n') {
                let complete_data = &buffer[..=last_newline_offset];
                let text = String::from_utf8_lossy(complete_data);
                let mut custom_log_data = CustomLogData::new();
                let mut new_lines = 0;

                for line in text.lines() {
                    let line = handle_line(line.to_string(), &mut custom_log_data);
                    for entry in line {
                        writeln!(writer, "{entry}").ok();
                        new_lines += 1;
                    }
                }
                pos += (last_newline_offset + 1) as u64;

                if new_lines > 0 {
                    let _ = window.emit("live_log_progress", new_lines);
                }
            }

            thread::sleep(Duration::from_secs(5));
        }
    });

    Ok(())
}

#[derive(PartialEq, Eq)]
enum PickerType {
    SingleFile,
    MultipleFiles,
    Folder,
    #[allow(dead_code)]
    MultipleFolders,
}

fn pick_files_internal(
    window: &Window,
    picker_type: PickerType,
    state: &State<'_, AppState>,
) -> Result<(), String> {
    let default_path = window
        .app_handle()
        .path()
        .resolve(
            "Elder Scrolls Online/live/logs/Encounter.log",
            BaseDirectory::Document,
        )
        .unwrap();
    let default_dir = default_path
        .parent()
        .unwrap_or_else(|| default_path.as_path());

    let dialog = window
        .dialog()
        .file()
        .add_filter("Encounter logs", &["log"])
        .set_directory(default_dir);

    let folder_dialog = window
        .dialog()
        .file()
        .set_directory(default_dir);

    if picker_type == PickerType::Folder {
        let log_tool_live = default_dir.join("LogToolLive");
        if log_tool_live.exists() {
            if let Err(e) = std::fs::remove_dir_all(&log_tool_live) {
                return Err(format!("Failed to delete LogToolLive folder: {}", e));
            }
        }
    }


    let picked_files = match picker_type {
        PickerType::SingleFile => dialog.blocking_pick_file().map(|f| vec![f]),
        PickerType::MultipleFiles => dialog.blocking_pick_files(),
        PickerType::Folder => folder_dialog.blocking_pick_folder().map(|f| vec![f]),
        PickerType::MultipleFolders => folder_dialog.blocking_pick_folders(),
    };

    match picker_type {
        PickerType::SingleFile | PickerType::MultipleFiles =>     
            if let Some(file_paths) = picked_files {
                let mut log_files_lock = state.log_files.write().map_err(|e| e.to_string())?;
                *log_files_lock = Some(file_paths.clone());
                Ok(())
            } else {
                let mut log_files_lock = state.log_files.write().map_err(|e| e.to_string())?;
                *log_files_lock = None;
                Err("No file(s) selected".to_string())
            },
        PickerType::Folder => if let Some(file_path) = picked_files {
                let mut log_files_lock = state.live_log_folder.write().map_err(|e| e.to_string())?;
                *log_files_lock = Some(file_path[0].clone());
                Ok(())
            } else {
                let mut log_files_lock = state.log_files.write().map_err(|e| e.to_string())?;
                *log_files_lock = None;
                Err("No folder selected".to_string())
            },
        _ => Ok(()),
    }
}

#[tauri::command]
fn pick_and_load_file(window: Window, state: State<'_, AppState>) -> Result<(), String> {
    pick_files_internal(&window, PickerType::SingleFile, &state)
}

#[tauri::command]
fn pick_and_load_files(window: Window, state: State<'_, AppState>) -> Result<(), String> {
    pick_files_internal(&window, PickerType::MultipleFiles, &state)
}

#[tauri::command]
fn pick_and_load_folder(window: Window, state: State<'_, AppState>) -> Result<(), String> {
    pick_files_internal(&window, PickerType::Folder, &state)
}

#[tauri::command]
fn delete_log_file(state: State<'_, AppState>) -> Result<(), String> {
    let paths_guard = state.log_files.read().unwrap();
    let file_paths = paths_guard.as_ref().ok_or("No file paths set")?;
    let file_path = file_paths.get(0).ok_or("No file path in vector")?;
    let path_ref = file_path.as_path().ok_or("Invalid file path")?;
    std::fs::remove_file(path_ref).map_err(|e| format!("Failed to delete file: {}", e))?;
    Ok(())
}

fn save_login_response(resp: &LoginResponse) {
    let path = cookie_file_path().with_file_name("login_response.json");
    if let Ok(json) = serde_json::to_string(resp) {
        log::info!("Saving login response");
        let _ = fs::write(path, json);
    }
}

fn load_login_response() -> Option<LoginResponse> {
    let path = cookie_file_path().with_file_name("login_response.json");
    if let Ok(data) = fs::read_to_string(path) {
        serde_json::from_str(&data).ok()
    } else {
        None
    }
}

#[tauri::command]
fn get_saved_login_response() -> Option<LoginResponse> {
    load_login_response()
}

fn save_upload_settings(resp: &UploadSettings) {
    let path = cookie_file_path().with_file_name("user-settings.json");
    if let Ok(json) = serde_json::to_string(&resp) {
        log::info!("Saving upload settings {}", json);
        let _ = fs::write(path, json);
    }
}

fn load_upload_settings() -> Option<UploadSettings> {
    let path = cookie_file_path().with_file_name("user-settings.json");
    if let Ok(data) = fs::read_to_string(path) {
        log::trace!("Returning saved settings {}", data);
        serde_json::from_str(&data).ok()
    } else {
        None
    }
}

#[tauri::command]
fn get_saved_upload_settings() -> Option<UploadSettings> {
    load_upload_settings()
}

#[tauri::command]
async fn login(state: tauri::State<'_, AppState>, username: String, password: String) -> Result<LoginResponse, String> {
    log::info!("Attempting to log in");
    let payload = serde_json::json!({ "email": username, "password": password, "version": ESO_LOGS_COM_VERSION });

    let client = {
        let client_guard = state.http.read().map_err(|e| e.to_string())?;
        client_guard.client.clone()
    };
    let resp = client
        .post("https://www.esologs.com/desktop-client/log-in")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Server returned {}", resp.status()));
    }
    log::debug!("{:?}", resp.headers());
    let text = resp.text().await.map_err(|e| format!("Failed to read response text: {e}"))?;
    let body: LoginResponse = serde_json::from_str(&text).map_err(|e| format!("Invalid JSON: {e}"))?;
    {
        let http = state.http.read().unwrap();
        let store = http.cookie_store.lock().unwrap();
        for cookie in store.iter_any() {
            log::debug!("{:?}", cookie);
        }
    }

    state.http.write().unwrap().save_cookies();
    save_login_response(&body);
    Ok(body)
}

#[tauri::command]
fn logout(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let http = state.http.write().map_err(|e| e.to_string())?;
    if let Ok(mut store) = http.cookie_store.lock() {
        store.clear();
        if let Ok(json) = serde_json::to_string(&*store) {
            let _ = fs::write(cookie_file_path(), json);
        }
    }
    let login_response_path = cookie_file_path().with_file_name("login_response.json");
    if let Err(e) = fs::remove_file(login_response_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            log::warn!("Failed to remove login_response.json: {}", e);
        }
    }
    let settings_path = cookie_file_path().with_file_name("user-settings.json");
    if let Err(e) = fs::remove_file(settings_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            log::warn!("Failed to remove user-settings.json: {}", e);
        }
    }
    Ok(())
}

#[tauri::command]
fn cancel_upload_log(state: State<'_, AppState>) -> Result<(), String> {
    state.upload_cancel_flag.store(true, SeqCst);
    Ok(())
}

async fn create_report(
    state: &State<'_, AppState>,
    client: &Client,
    settings: &UploadSettings,
) -> Result<EncounterReportCode, String> {
    state.upload_cancel_flag.store(false, SeqCst);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    let payload = json!({
        "clientVersion": ESO_LOGS_COM_VERSION,
        "parserVersion": ESO_LOGS_PARSER_VERSION,
        "startTime": now,
        "endTime": now,
        "fileName": "Encounter.log",
        "serverOrRegion": settings.region,
        "visibility": settings.visibility,
        "reportTagId": null,
        "description": settings.description,
        "guildId": if settings.guild == -1 { None } else { Some(settings.guild) },
    });
    log::debug!("Create-report payload: {payload}");

    log::info!("POST https://www.esologs.com/desktop-client/create-report");

    let response = client
        .post("https://www.esologs.com/desktop-client/create-report")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Request error: {e}"))?;

    let status = response.status();
    let raw_body = response.text().await.map_err(|e| format!("Failed to read response text: {e}"))?;

    log::trace!("Received response status: {}", status);
    log::trace!("Raw response body: {}", raw_body);

    if !status.is_success() {
        return Err(format!("Server returned error status: {} with body: {}", status, raw_body));
    }

    state.http.write().unwrap().save_cookies();

    let report: EncounterReportCode = serde_json::from_str(&raw_body)
        .map_err(|e| format!("Invalid JSON: {e}\nRaw body: {raw_body}"))?;

    log::info!("Parsed report: {:?}", report);

    let code = report.code.clone();
    *state.esolog_code.write().map_err(|e| e.to_string())? = Some(code.clone());
    Ok(report)
}

#[tauri::command]
async fn upload_log(window: Window, state: State<'_, AppState>, upload_settings: UploadSettings) -> Result<EncounterReportCode, String> {
    log::info!("Beginning direct log upload process");
    state.upload_cancel_flag.store(false, SeqCst);
    let log_path_opt = {
        let lock = state.log_files.read().map_err(|e| e.to_string())?;
        log::debug!("log_files = {:?}", *lock);
        lock.clone()
    };
    let log_path = log_path_opt
        .and_then(|v| v.get(0).cloned())
        .ok_or("No log file selected")?;
    log::debug!("Using log file: {:?}", log_path);

    let client = {
        let g = state.http.read().map_err(|e| e.to_string())?;
        g.client.clone()
    };
    log::info!("Spawning split/zip task ...");
    
    let tmp_dir = temp_dir().join("esologtool_temporary");
    log::trace!("Temp dir: {:?}", tmp_dir);
    create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;

    let tmp_dir_for_spawn = tmp_dir.clone();
    let log_path_clone = log_path.clone();
    let upload_cancel_flag = state.upload_cancel_flag.clone();

    let window_clone = window.clone();
    let pairs = spawn_blocking(move || -> Result<Vec<(PathBuf,u16)>, String> {
        if upload_cancel_flag.load(SeqCst) {
            return Err("Upload cancelled".to_string());
        }
        split_and_zip_log_by_fight(
            log_path_clone.as_path().ok_or("Invalid log file path")?,
            tmp_dir_for_spawn.as_path(),
            |val| {
                let _ = window_clone.emit("upload_progress", format!("Processing: {}%", val));
            },
        )?;
        log::debug!("Finished split_and_zip_log_by_fight");

        let mut out = Vec::new();
        for idx in 1u16.. {
            let seg = tmp_dir_for_spawn.join(format!("report_segment_{idx}.zip"));
            if seg.exists() {
                out.push((seg, idx));
            } else {
                break;
            }
        }
        Ok(out)
    }).await.map_err(|e| format!("spawn_blocking error: {e}"))??;

    let _ = window.emit("upload_progress", format!("Processing: 100%"));

    if state.upload_cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
        return Err("Upload cancelled".to_string());
    }
    let report_code = create_report(&state, &client, &upload_settings).await?;
    let code = report_code.code.clone();

    let base = "https://www.esologs.com/desktop-client";
    let mut segment_id = 1u16;

    let ts_path = tmp_dir.join("timestamps");
    let timestamps   = read_timestamps(&ts_path)?;

    let total_segments = pairs.len();
    let mut uploaded_segments = 0;
    
    window
        .emit("live_log_code", code.clone())
        .map_err(|e| format!("Failed to emit uploading log code: {}", e))?;

    let last_idx = pairs.len().saturating_sub(1);
    log::info!("Uploading segments on main thread ... ");
    for ((seg, _), (start, end)) in pairs.iter().zip(timestamps.iter()) {
        if state.upload_cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
            upload_master_table( 
                &client,
                &format!("{base}/set-report-master-table/{code}"),
                last_idx.try_into().unwrap(),
                &tmp_dir.join(format!("master_table.zip")).clone(),
            ).await?;
            log::info!("Uploading master table ... ");
            return Err("Upload cancelled".to_string());
        }
        segment_id = upload_segment_and_get_next_id(
            &client,
            &format!("{base}/add-report-segment/{code}"),
            seg,
                segment_id, 
                *start, 
                *end,
        ).await?;
        uploaded_segments += 1;
        let _ = window.emit("upload_progress", format!("Uploading: {}%", ((uploaded_segments as f64 / total_segments as f64) * 100.0).round() as u8)); 
    }
    log::info!("Uploading master table ... ");
        upload_master_table( 
        &client,
        &format!("{base}/set-report-master-table/{code}"),
        segment_id,
        &tmp_dir.join(format!("master_table.zip")).clone(),
    ).await?;  
    let _ = window.emit("upload_progress", format!("Uploading: 100%"));
    log::trace!("POST {base}/terminate-report/{code}");
    client.post(&format!("{base}/terminate-report/{code}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    log::info!("Report terminated OK");

    end_report(&client, code.clone()).await;

    // if let Err(e) = fs::remove_dir_all(&tmp_dir) {
    //     log::warn!("Failed to remove temp dir {:?}: {}", tmp_dir, e);
    // }
    
    save_upload_settings(&upload_settings);

    Ok(report_code)
}

async fn upload_master_table(
    client: &reqwest::Client,
    url: &str,
    segment_id: u16,
    zip_path: &Path,
) -> Result<(), String> {
    log::trace!("→ upload_master_table(): segment_id = {segment_id}");
    log::trace!("  ZIP path = {:?}", zip_path);
    log::trace!("  POST {}", url);

    let bytes = fs::read(zip_path)
        .map_err(|e| format!("Failed to read master_table zip: {e}"))?;
    log::trace!("  size = {} bytes", bytes.len());

    let part = Part::bytes(bytes)
        .file_name(zip_path.file_name().unwrap().to_string_lossy().to_string())
        .mime_str("application/zip")
        .map_err(|e| format!("Invalid MIME type: {e}"))?;

    let form = Form::new()
        .text("segmentId", segment_id.to_string())
        .text("isRealTime", "false")
        .part("logfile", part);

    let resp = client.post(url).multipart(form).send()
        .await
        .map_err(|e| format!("Request error: {e}"))?;

    let status = resp.status();

    log::trace!("  status = {}", status);

    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        log::trace!("  body   = {body}");
        return Err(format!("Master table upload failed: {} - {body}", status));
    }

    log::trace!("  ✔ master table upload OK");
    Ok(())
}

async fn upload_segment_and_get_next_id(
    client: &reqwest::Client,
    url: &str,
    zip_path: &Path,
    segment_id: u16,
    start_time: u64,
    end_time: u64,
) -> Result<u16, String> {
    log::trace!("→ upload_segment_and_get_next_id()");
    log::trace!("  segment_id = {segment_id}");
    log::trace!("  ZIP path   = {:?}", zip_path);
    log::trace!("  POST       = {url}");

    let bytes = fs::read(zip_path)
        .map_err(|e| format!("Failed to read segment zip: {e}"))?;
    log::trace!("  size       = {} bytes", bytes.len());

    let params = serde_json::json!({
        "startTime":            start_time,
        "endTime":              end_time,
        "mythic":               0,
        "isLiveLog":            false,
        "isRealTime":           false,
        "inProgressEventCount": 0,
        "segmentId":            segment_id,
    });
    log::trace!("{}", params);

    let logfile_part = Part::bytes(bytes)
        .file_name(zip_path.file_name().unwrap().to_string_lossy().to_string())
        .mime_str("application/zip")
        .map_err(|e| format!("Invalid MIME type: {e}"))?;

    let parameters_part = Part::text(params.to_string())
        .mime_str("application/json")
        .map_err(|e| format!("Invalid MIME type: {e}"))?;

    let form = Form::new()
        .part("logfile", logfile_part)
        .part("parameters", parameters_part);

    let resp = client.post(url).multipart(form).send()
        .await
        .map_err(|e| format!("Request error: {e}"))?;

    let status = resp.status();
    let body   = resp.text().await
        .map_err(|e| format!("Failed to read response text: {e}"))?;

    log::trace!("  status     = {}", status);
    log::trace!("  raw body   = {}", body);

    if !status.is_success() {
        return Err(format!("Segment upload failed: {status} - {body}"));
    }

    let next_id = serde_json::from_str::<serde_json::Value>(&body)
        .map_err(|e| format!("Bad JSON: {e}\nRaw body: {body}"))?
        .get("nextSegmentId")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("Missing `nextSegmentId` in response: {body}"))?;

    log::trace!("  ✔ nextSegmentId = {next_id}");
    Ok(next_id as u16)
}

async fn end_report(client: &reqwest::Client, code: String) {
    log::debug!("POST https://www.esologs.com/desktop-client/terminate-report");

    let _response = client
        .post(format!("https://www.esologs.com/desktop-client/terminate-report/{}", code))
        .send()
        .await
        .map_err(|e| format!("Request error: {e}"));
}

fn read_timestamps(path: &Path) -> Result<Vec<(u64, u64)>, String> {
    use std::io::{BufRead, BufReader};
    let f = File::open(path)
        .map_err(|e| format!("Failed to open {path:?}: {e}"))?;
    let mut out = Vec::new();

    for (i, line) in BufReader::new(f).lines().enumerate() {
        let line = line.map_err(|e| format!("Read error at line {i}: {e}"))?;
        let mut split = line.splitn(2, ',');
        let start = split.next()
            .ok_or("Missing startTime")?
            .parse::<u64>()
            .map_err(|e| format!("Bad startTime at line {i}: {e}"))?;
        let end = split.next()
            .ok_or("Missing endTime")?
            .parse::<u64>()
            .map_err(|e| format!("Bad endTime at line {i}: {e}"))?;
        out.push((start, end));
    }
    Ok(out)
}

#[tauri::command]
async fn live_log_upload(window: Window, app_state: State<'_, AppState>, upload_settings: UploadSettings) -> Result<EncounterReportCode, String> {
    log::info!("Beginning direct live log upload ...");
    let input_path: PathBuf = {
        let guard = app_state.live_log_folder.read().map_err(|e| e.to_string())?;
        let folder = guard.as_ref().ok_or("No folder selected")?.clone();
        folder.as_path().ok_or("Invalid path")?.join("Encounter.log")
    };
    {
        let mut log_files_lock = app_state.log_files.write().map_err(|e| e.to_string())?;
        *log_files_lock = Some(vec![FilePath::from(input_path.clone())]);
    }

    log::trace!("[live_log_upload] Using input path: {:?}", input_path);

    let client = {
        let g = app_state.http.read().map_err(|e| e.to_string())?;
        g.client.clone()
    };

    log::trace!("[live_log_upload] Creating report...");
    let report_code = create_report(&app_state, &client, &upload_settings).await?;
    let code: String = report_code.code.clone();
    log::trace!("[live_log_upload] Report code: {}", code);

    window
        .emit("live_log_code", code.clone())
        .map_err(|e| format!("Failed to emit live log code: {}", e))?;

    let base = "https://www.esologs.com/desktop-client".to_string();
    let tmp_dir = std::env::temp_dir().join(format!("esologtool_live_{code}"));
    std::fs::create_dir_all(&tmp_dir)
        .map_err(|e| format!("Failed to create temp dir: {e}"))?;
    log::trace!("[live_log_upload] Temp dir created at {:?}", tmp_dir);

    let window = window.clone();
    let upload_cancel_flag = app_state.upload_cancel_flag.clone();
    let handle = tauri::async_runtime::spawn(async move {
        log::trace!("[live_log_upload] Spawned async task.");

        let mut input_file = loop {
            match std::fs::OpenOptions::new().read(true).open(&input_path) {
                Ok(f) => {
                    log::trace!("[live_log_upload] Successfully opened Encounter.log");
                    break f;
                }
                Err(e) => {
                    log::warn!(
                        "[live_log_upload] Failed to open Encounter.log ({}). Retrying...",
                        e
                    );
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    continue;
                }
            }
        };

        let mut pos = if upload_settings.rewind {
            log::trace!("[live_log_upload] Rewind enabled → seeking to start of file.");
            input_file.seek(std::io::SeekFrom::Start(0)).expect("seek failed")
        } else {
            log::trace!("[live_log_upload] Rewind disabled → seeking to end of file.");
            let scan_file = std::fs::OpenOptions::new().read(true).open(&input_path)
                .expect("Failed to open file for scanning");
            let mut reader = std::io::BufReader::new(&scan_file);
            let mut pos = 0u64;
            let mut last_begin_log_pos = 0u64;
            loop {
                let start_pos = pos;
                let mut buf = String::new();
                let bytes_read = reader.read_line(&mut buf).expect("read failed");
                if bytes_read == 0 {
                    break;
                }
                if buf.contains("BEGIN_LOG") {
                    last_begin_log_pos = start_pos;
                }
                pos += bytes_read as u64;
            }
            input_file.seek(std::io::SeekFrom::Start(last_begin_log_pos)).expect("seek failed")
        };
        log::trace!("[live_log_upload] Initial file position: {}", pos);

        let mut elp = ESOLogProcessor::new();
        let mut custom_state = CustomLogData::new();
        let mut first_timestamp: Option<u64> = None;
        let mut segment_id: u16 = 1;
        let mut processed = 0usize;

        loop {
            if upload_cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
                log::info!("[live_log_upload] Cancel flag set → breaking loop.");
                break;
            }

            input_file
                .seek(std::io::SeekFrom::Start(pos))
                .expect("seek failed");

            let mut buffer = Vec::new();
            let mut reader = std::io::BufReader::new(&input_file);
            let bytes_read = reader.read_to_end(&mut buffer).expect("read failed");

            log::trace!("[live_log_upload] Read {} bytes at pos {}", bytes_read, pos);

            if bytes_read == 0 {
                log::trace!("[live_log_upload] No new bytes. Sleeping 5s...");
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }

            if let Some(last_nl) = buffer.iter().rposition(|&b| b == b'\n') {
                let text = String::from_utf8_lossy(&buffer[..=last_nl]);

                for line in text.lines() {
                    let mut split = line.splitn(4, ',');
                    let _first = split.next();
                    let second = split.next();
                    let third = split.next();

                    if matches!(second, Some("BEGIN_LOG")) {
                        log::trace!("[live_log_upload] BEGIN_LOG encountered.");
                        elp.eso_logs_log.new_log_reset();
                        elp.reset();
                        custom_state.reset();
                        if let Some(third_str) = third {
                            if let Ok(ts) = third_str.parse::<u64>() {
                                log::trace!(
                                    "[live_log_upload] BEGIN_LOG timestamp = {}",
                                    ts
                                );
                                if first_timestamp.is_none() {first_timestamp = Some(ts)};
                            }
                        }
                    }

                    let is_end_log = matches!(second, Some("END_LOG"));
                    let is_end_combat = matches!(second, Some("END_COMBAT"));
                    if is_end_combat {
                        log::trace!("[live_log_upload] END_COMBAT encountered.");
                    }
                    if is_end_log {
                        log::trace!("[live_log_upload] END_LOG encountered.");
                    }

                    for l in handle_line(line.to_string(), &mut custom_state) {
                        elp.handle_line(l.to_string());
                    }

                    if is_end_combat {
                        log::trace!(
                            "[live_log_upload] Packaging segment {}...",
                            segment_id
                        );

                        let seg_zip =
                            tmp_dir.join(format!("report_segment_{segment_id}.zip"));
                        let tbl_zip =
                            tmp_dir.join(format!("master_table.zip"));

                        let seg_data = build_report_segment(&elp);
                        write_zip_with_logtxt(&seg_zip, seg_data.as_bytes())
                            .expect("seg zip write failed");

                        let tbl_data = build_master_table(&mut elp);
                        write_zip_with_logtxt(&tbl_zip, tbl_data.as_bytes())
                            .expect("tbl zip write failed");

                        let (start_ts, end_ts) = {
                            let events = &elp.eso_logs_log.events;
                            if !events.is_empty() {
                                let mut last_ts = event_timestamp(&events[events.len() - 1]);
                                if last_ts.is_some() && first_timestamp.is_some() {
                                    last_ts = Some(last_ts.unwrap() + first_timestamp.unwrap());
                                }
                                match (first_timestamp, last_ts) {
                                    (Some(first), Some(last)) => (first, last),
                                    _ => (0, 0),
                                }
                            } else {
                                (0, 0)
                            }
                        };
                        log::trace!(
                            "[live_log_upload] Segment {} time range: start={} end={}",
                            segment_id, start_ts, end_ts
                        );

                        if let Err(e) = upload_master_table(
                            &client,
                            &format!("{base}/set-report-master-table/{code}"),
                            segment_id,
                            &tbl_zip,
                        ).await {
                            log::error!("Master table upload failed: {e}");
                        }

                        match upload_segment_and_get_next_id(
                            &client,
                            &format!("{base}/add-report-segment/{code}"),
                            &seg_zip,
                            segment_id,
                            start_ts,
                            end_ts,
                        ).await {
                            Ok(next) => {
                                log::trace!(
                                    "[live_log_upload] Segment {} uploaded. Next = {}",
                                    segment_id, next
                                );
                                segment_id = next;
                            }
                            Err(e) => {
                                log::error!("Segment upload failed: {e}");
                                segment_id += 1;
                            }
                        }

                        elp.eso_logs_log.events.clear();
                        custom_state.reset();
                        let _ = window.emit("upload_progress", format!("Total lines processed: {}", processed));
                    }

                    processed += 1;
                }

                if processed > 0 {
                    log::trace!("[live_log_upload] Processed lines so far: {}", processed);
                }

                pos += (last_nl + 1) as u64;
                log::trace!("[live_log_upload] New file position: {}", pos);
            }

            std::thread::sleep(std::time::Duration::from_secs(5));
        }

        log::trace!("[live_log_upload] Loop exited. Sending terminate-report...");

        let _ = client
            .post(&format!("{base}/terminate-report/{code}"))
            .send()
            .await;

        let _ = std::fs::remove_dir_all(&tmp_dir);
        log::trace!("[live_log_upload] Temp dir deleted. Task exiting.");
    });

    if let Err(e) = handle.await {
        return Err(format!("Live log task failed: {e}"));
    }

    log::trace!("[live_log_upload] Upload settings saved.");
    save_upload_settings(&upload_settings);

    Ok(report_code)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = Ftail::new()
        .console(LevelFilter::Debug)
        .daily_file({
            let mut path = cookie_folder_path();
            path.push("logs");
            if let Err(e) = std::fs::create_dir_all(&path) {
                log::warn!("Failed to create logs folder: {}", e);
            }
            &path.clone()
        }, LevelFilter::Info)
    .init();
    match result {
        Ok(_) => log::info!("Logging initialised"),
        Err(e) => println!("Error initialising logging: {}", e),
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                update(handle).await.unwrap();
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            pick_and_load_file,
            pick_and_load_files,
            pick_and_load_folder,
            modify_log_file,
            split_encounter_file_into_log_files,
            combine_encounter_log_files,
            live_log_from_folder,
            login,
            logout,
            upload_log,
            live_log_upload,
            cancel_upload_log,
            delete_log_file,
            get_saved_login_response,
            get_saved_upload_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
  if let Some(update) = app.updater()?.check().await? {
    let mut downloaded = 0;

    update
      .download_and_install(
        |chunk_length, content_length| {
          downloaded += chunk_length;
          log::info!("downloaded {downloaded} from {content_length:?}");
        },
        || {
          log::info!("download finished");
        },
      )
      .await?;

    log::info!("update installed");
    // app.restart();
  }

  Ok(())
}