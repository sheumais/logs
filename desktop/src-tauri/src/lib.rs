use std::{fs::{File, OpenOptions}, io::{BufRead, BufReader, BufWriter, Write}};
use cli::{log_edit::check_line_for_edits, read_file};
use state::AppState;
use tauri::{path::BaseDirectory, Emitter, Manager, State, Window};
use tauri_plugin_dialog::{DialogExt, FilePath};
mod state;

fn load_file(path: FilePath, state: State<'_, AppState>) -> Result<(), String> {
    let path_ref = path.as_path().ok_or("Invalid file path")?;
    let logs = read_file(path_ref).map_err(|e| e.to_string())?;

    let mut stored_logs = state.logs.write().map_err(|e| e.to_string())?;
    *stored_logs = Some(logs);
    Ok(())
}

#[tauri::command]
fn modify_log_file(window: Window, state: State<'_, AppState>) -> Result<(), String> {
    let path_guard = state.log_file.read().unwrap();
    let file_path = path_guard.as_ref().ok_or("Invalid file path")?;
    let path_ref = file_path.as_path().ok_or("Invalid file path")?;
    let file = File::open(path_ref).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(&file);
    let total_lines = reader.lines().count();
    let file = File::open(path_ref).map_err(|e| format!("Failed to reopen file: {}", e))?;
    let reader = BufReader::new(file);

    let mut new_path = path_ref.to_path_buf();
    new_path.set_extension("");
    if let Some(stem) = new_path.file_stem() {
        let mut new_file_name = stem.to_os_string();
        new_file_name.push("-MODIFIED.log");
        new_path.set_file_name(new_file_name);
    } else {
        return Err("Failed to get file stem".into());
    }

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&new_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut writer = BufWriter::new(file);

    let mut processed = 0;
    for line_result in reader.lines() {
        let line = line_result.map_err(|e| format!("Error reading line: {}", e))?;

        let mut in_brackets = false;
        let mut current_segment_start = 0;
        let mut parts = Vec::new();

        for (i, char) in line.char_indices() {
            match char {
                '[' => {
                    in_brackets = true;
                    current_segment_start = i + 1;
                }
                ']' => {
                    in_brackets = false;
                    parts.push(&line[current_segment_start..i]);
                    current_segment_start = i + 1;
                }
                ',' if !in_brackets => {
                    parts.push(&line[current_segment_start..i]);
                    current_segment_start = i + 1;
                }
                _ => {}
            }
        }

        if current_segment_start < line.len() {
            parts.push(&line[current_segment_start..]);
        }
        parts.retain(|part| !part.is_empty());

        let parts_clone = parts.clone();
        let new_addition = check_line_for_edits(parts);
        if let Some(new_lines) = new_addition {
            for new_line in new_lines {
                writeln!(writer, "{}", new_line)
                    .map_err(|e| format!("Failed to write new modified line: {}", e))?;
            }
            if parts_clone.get(1) != Some(&"ABILITY_INFO") && parts_clone.get(1) != Some(&"EFFECT_INFO") {
                writeln!(writer, "{}", line)
                    .map_err(|e| format!("Failed to write original line: {}", e))?;
            }
        } else {
            writeln!(writer, "{}", line)
                .map_err(|e| format!("Failed to write original line: {}", e))?;
        }

        processed += 1;
        if processed % 25000 == 0 || processed == total_lines {
            let progress = (processed * 100 / total_lines).min(100);
            window
                .emit("log_modify_progress", progress)
                .map_err(|e| format!("Failed to emit progress: {}", e))?;
        }
    }

    Ok(())
}

#[tauri::command]
fn pick_and_load_file(window: Window, state: State<'_, AppState>) -> Result<(), String> {
    let default_path = window
        .app_handle()
        .path()
        .resolve("Elder Scrolls Online/live/logs/Encounter.log", BaseDirectory::Document)
        .unwrap();
    let default_dir = default_path.parent().unwrap_or_else(|| default_path.as_path());
    if let Some(file_path) = window.dialog().file()
        .add_filter("Encounter logs", &["log"])
        .set_directory(default_dir).blocking_pick_file() {
        let mut log_file_lock = state.log_file.write().map_err(|e| e.to_string())?;
        *log_file_lock = Some(file_path.clone());
        Ok(())
    } else {
        Err("No file selected".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![pick_and_load_file, modify_log_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
