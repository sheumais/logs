use cli::log_edit::check_line_for_edits;
use state::AppState;
use tauri_plugin_updater::UpdaterExt;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};
use tauri::{path::BaseDirectory, Emitter, Manager, State, Window};
use tauri_plugin_dialog::DialogExt;
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
            if parts_clone.get(1) != Some(&"ABILITY_INFO")
                && parts_clone.get(1) != Some(&"EFFECT_INFO")
                && parts_clone.get(1) != Some(&"PLAYER_INFO")
            {
                writeln!(writer, "{}", line)
                    .map_err(|e| format!("Failed to write original line: {}", e))?;
            }
        } else {
            writeln!(writer, "{}", line)
                .map_err(|e| format!("Failed to write original line: {}", e))?;
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
fn split_encounter_file_into_log_files(
    window: Window,
    state: State<'_, AppState>,
) -> Result<(), String> {
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
            let out_name = format!("Split-encounter-{}.log", time);
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
        "Combined-encounter-{}-{}.log",
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

fn pick_files_internal(
    window: &Window,
    allow_multiple: bool,
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

    let picked_files = if allow_multiple {
        dialog.blocking_pick_files()
    } else {
        dialog.blocking_pick_file().map(|f| vec![f])
    };

    if let Some(file_paths) = picked_files {
        let mut log_files_lock = state.log_files.write().map_err(|e| e.to_string())?;
        *log_files_lock = Some(file_paths.clone());
        Ok(())
    } else {
        let mut log_files_lock = state.log_files.write().map_err(|e| e.to_string())?;
        *log_files_lock = None;
        Err("No file(s) selected".to_string())
    }
}

#[tauri::command]
fn pick_and_load_file(window: Window, state: State<'_, AppState>) -> Result<(), String> {
    pick_files_internal(&window, false, &state)
}

#[tauri::command]
fn pick_and_load_files(window: Window, state: State<'_, AppState>) -> Result<(), String> {
    pick_files_internal(&window, true, &state)
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
            modify_log_file,
            split_encounter_file_into_log_files,
            combine_encounter_log_files
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