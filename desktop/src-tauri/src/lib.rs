use cli::{esologs_convert::split_and_zip_log_by_fight, esologs_format::{EncounterReportCode, LoginResponse, ESO_LOGS_COM_VERSION, ESO_LOGS_PARSER_VERSION}, log_edit::{handle_line, CustomLogData}};
use reqwest::multipart::{Form, Part};
use state::AppState;
use tauri_plugin_updater::UpdaterExt;
use std::{
    fs::{self, File, OpenOptions}, io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}, thread, time::Duration
};
use tauri::{path::BaseDirectory, Emitter, Manager, State, Window};
use tauri_plugin_dialog::DialogExt;

use crate::state::cookie_file_path;
mod state;

const LINE_COUNT_FOR_PROGRESS: usize = 25000usize;

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
                eprintln!("Error reading line: {}", e);
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

fn save_login_response(resp: &LoginResponse) {
    let path = cookie_file_path().with_file_name("login_response.json");
    if let Ok(json) = serde_json::to_string(resp) {
        println!("Saving login response");
        let _ = fs::write(path, json);
    }
}

fn load_login_response() -> Option<LoginResponse> {
    let path = cookie_file_path().with_file_name("login_response.json");
    if let Ok(data) = fs::read_to_string(path) {
        println!("Loading login response");
        serde_json::from_str(&data).ok()
    } else {
        None
    }
}

#[tauri::command]
fn get_saved_login_response() -> Option<LoginResponse> {
    load_login_response()
}

#[tauri::command]
async fn login(
    state: tauri::State<'_, AppState>,
    username: String,
    password: String,
) -> Result<LoginResponse, String> {
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
    println!("{:?}", resp.headers());
    let text = resp.text().await.map_err(|e| format!("Failed to read response text: {e}"))?;
    let body: LoginResponse = serde_json::from_str(&text).map_err(|e| format!("Invalid JSON: {e}"))?;
    {
        let http = state.http.read().unwrap();
        let store = http.cookie_store.lock().unwrap();
        for cookie in store.iter_any() {
            println!("{:?}", cookie);
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
            println!("Failed to remove login_response.json: {}", e);
        }
    }
    Ok(())
}
#[tauri::command]
async fn upload_log(
    state: State<'_, AppState>
) -> Result<EncounterReportCode, String> {
    let log_path_opt = {
        let lock = state.log_files.read().map_err(|e| e.to_string())?;
        println!("log_files = {:?}", *lock);
        lock.clone()
    };
    let log_path = log_path_opt
        .and_then(|v| v.get(0).cloned())
        .ok_or("No log file selected")?;
    println!("Using log file: {:?}", log_path);

    let file = File::open(log_path.as_path().ok_or("Invalid log file path")?).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    reader.read_line(&mut first_line).map_err(|e| e.to_string())?;
    let start_timestamp = first_line
        .splitn(4, ',')
        .nth(2)
        .ok_or("Malformed BEGIN_LOG line")?
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse timestamp: {e}"))?;

    let time = start_timestamp;
    let payload = serde_json::json!({
        "clientVersion": ESO_LOGS_COM_VERSION,
        "parserVersion": ESO_LOGS_PARSER_VERSION,
        "startTime": time,
        "endTime": time,
        "fileName": "Encounter.log",
        "serverOrRegion": 1,
        "visibility": 2,
        "reportTagId": null,
        "description": "",
        "guildId": null
    });
    println!("Create-report payload: {payload}");

    let client = {
        let g = state.http.read().map_err(|e| e.to_string())?;
        g.client.clone()
    };
    println!("Spawning split/zip task …");
    
    let tmp_dir = std::env::temp_dir().join("esologtool_temporary");
    println!("Temp dir: {:?}", tmp_dir);
    fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;

    let tmp_dir_for_spawn = tmp_dir.clone();
    let log_path_clone = log_path.clone();

    let pairs = tauri::async_runtime::spawn_blocking(move || -> Result<Vec<(PathBuf,PathBuf,u16)>, String> {
        split_and_zip_log_by_fight(
            log_path_clone.as_path().ok_or("Invalid log file path")?,
            tmp_dir_for_spawn.as_path()
        )?;
        println!("Finished split_and_zip_log_by_fight");

        let mut out = Vec::new();
        for idx in 1u16.. {
            let tbl = tmp_dir_for_spawn.join(format!("master_table_{idx}.zip"));
            let seg = tmp_dir_for_spawn.join(format!("report_segment_{idx}.zip"));
            if tbl.exists() && seg.exists() {
                println!("Found pair idx={idx}: {:?}, {:?}", tbl, seg);
                out.push((tbl, seg, idx));
            } else {
                break;
            }
        }
        Ok(out)
    })
    .await
    .map_err(|e| format!("spawn_blocking error: {e}"))??;

    println!("POST https://www.esologs.com/desktop-client/create-report");

    let response = client
        .post("https://www.esologs.com/desktop-client/create-report")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Request error: {e}"))?;

    let status = response.status();
    let raw_body = response.text().await.map_err(|e| format!("Failed to read response text: {e}"))?;

    println!("Received response status: {}", status);
    println!("Raw response body: {}", raw_body);

    if !status.is_success() {
        return Err(format!("Server returned error status: {} with body: {}", status, raw_body));
    }

    state.http.write().unwrap().save_cookies();

    let report: EncounterReportCode = serde_json::from_str(&raw_body)
        .map_err(|e| format!("Invalid JSON: {e}\nRaw body: {raw_body}"))?;

    println!("Parsed report: {:?}", report);

    let code = report.code.clone();
    *state.esolog_code.write().map_err(|e| e.to_string())? = Some(code.clone());

    let base = "https://www.esologs.com/desktop-client";
    let mut segment_id = 1u16;

    let ts_path = tmp_dir.join("timestamps");
    let timestamps   = read_timestamps(&ts_path)?;

    for ((tbl, seg, _), (start, end)) in pairs.iter().zip(timestamps.iter()) {
        upload_master_table(
            &client,
            &format!("{base}/set-report-master-table/{code}"),
            segment_id,
            tbl,
        ).await?;
        segment_id = upload_segment_and_get_next_id(
            &client,
            &format!("{base}/add-report-segment/{code}"),
            seg,
            segment_id,
            *start,
            *end,
        ).await?;
    }

    println!("POST {base}/terminate-report/{code}");
    client.post(&format!("{base}/terminate-report/{code}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    state.http.write().unwrap().save_cookies();
    println!("Report terminated OK");

    end_report(&client, report.code.clone()).await;

    Ok(report)
}

async fn upload_master_table(
    client: &reqwest::Client,
    url: &str,
    segment_id: u16,
    zip_path: &Path,
) -> Result<(), String> {
    println!("→ upload_master_table(): segment_id = {segment_id}");
    println!("  ZIP path = {:?}", zip_path);
    println!("  POST {}", url);

    let bytes = fs::read(zip_path)
        .map_err(|e| format!("Failed to read master_table zip: {e}"))?;
    println!("  size = {} bytes", bytes.len());

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

    println!("  status = {}", status);

    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        println!("  body   = {body}");
        return Err(format!("Master table upload failed: {} – {body}", status));
    }

    println!("  ✔ master table upload OK");
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
    println!("→ upload_segment_and_get_next_id()");
    println!("  segment_id = {segment_id}");
    println!("  ZIP path   = {:?}", zip_path);
    println!("  POST       = {url}");

    let bytes = fs::read(zip_path)
        .map_err(|e| format!("Failed to read segment zip: {e}"))?;
    println!("  size       = {} bytes", bytes.len());

    let params = serde_json::json!({
        "startTime":            start_time,
        "endTime":              end_time,
        "mythic":               0,
        "isLiveLog":            false,
        "isRealTime":           false,
        "inProgressEventCount": 0,
        "segmentId":            segment_id,
    });
    println!("{}", params);

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

    println!("  status     = {}", status);
    println!("  raw body   = {}", body);

    if !status.is_success() {
        return Err(format!("Segment upload failed: {status} – {body}"));
    }

    let next_id = serde_json::from_str::<serde_json::Value>(&body)
        .map_err(|e| format!("Bad JSON: {e}\nRaw body: {body}"))?
        .get("nextSegmentId")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("Missing `nextSegmentId` in response: {body}"))?;

    println!("  ✔ nextSegmentId = {next_id}");
    Ok(next_id as u16)
}

async fn end_report(client: &reqwest::Client, code: String) {
    println!("POST https://www.esologs.com/desktop-client/terminate-report");

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
            get_saved_login_response
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
          println!("downloaded {downloaded} from {content_length:?}");
        },
        || {
          println!("download finished");
        },
      )
      .await?;

    println!("update installed");
    app.restart();
  }

  Ok(())
}