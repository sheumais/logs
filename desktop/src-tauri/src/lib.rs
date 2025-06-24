use cli::log_edit::{handle_line, ZenDebuffState};
use state::AppState;
use tauri_plugin_updater::UpdaterExt;
use std::{
    collections::HashMap, fs::{File, OpenOptions}, io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write}, path::Path, thread, time::Duration
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
    let mut zen_status: HashMap<u32, ZenDebuffState> = HashMap::new();

    let mut processed = 0;
    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                continue;
            }
        };
        let modified_line = handle_line(line, &mut zen_status);
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
                let mut zen_status: HashMap<u32, ZenDebuffState> = HashMap::new();
                let mut new_lines = 0;

                for line in text.lines() {
                    let line = handle_line(line.to_string(), &mut zen_status);
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
        PickerType::SingleFile => dialog.blocking_pick_files(),
        PickerType::MultipleFiles => dialog.blocking_pick_file().map(|f| vec![f]),
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
            live_log_from_folder
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