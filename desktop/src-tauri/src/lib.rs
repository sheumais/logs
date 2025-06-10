use std::{fs::File, io::{self, BufRead, BufReader}, path::{Path, PathBuf}};
use parser::{effect::Effect, log::Log};
use state::AppState;
use tauri::{path::BaseDirectory, Manager, State};
mod state;


pub fn read_file(file_path: &Path) -> io::Result<Vec<Log>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines().peekable();

    let mut logs = Vec::new();
    let mut current_log = Log::new();

    while let Some(line) = lines.next() {
        let line = line?;
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
        let second_value = parts[1];

        match second_value {
            "BEGIN_LOG" => {
                if !current_log.is_empty() {
                    logs.push(current_log);
                }
                current_log = Log::new();
                current_log.parse_line(parts);
            }
            "END_LOG" => {
                logs.push(current_log);
                current_log = Log::new();
            }
            _ => {
                current_log.parse_line(parts);
            }
        }
    }

    if !current_log.is_empty() {
        logs.push(current_log);
    }

    Ok(logs)
}

pub fn load_file(path: PathBuf, state: State<'_, AppState>) -> Result<(), String> {
    let logs = crate::read_file(&path).map_err(|e| e.to_string())?;

    let mut stored_logs = state.logs.write().map_err(|e| e.to_string())?;
    *stored_logs = Some(logs);
    Ok(())
}

#[tauri::command]
fn query_fights(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let stored_logs = state.logs.read().map_err(|e| e.to_string())?;
    let logs = stored_logs.as_ref().ok_or("No logs loaded")?;

    let mut output = Vec::new();
    for log in logs {
        for fight in &log.fights {
            let duration_secs = (fight.end_time - fight.start_time) / 1000;
            let minutes = duration_secs / 60;
            let seconds = duration_secs % 60;
            let boss_health_opt = fight.get_average_boss_health_percentage();
            let line = if let Some(boss_health) = boss_health_opt {
                if boss_health == 0.0 {
                    format!("{:2} - {} ({}:{:02}) KILL", fight.id, fight.name, minutes, seconds)
                } else {
                    format!("{:2} - {} ({}:{:02}) {:.0}%", fight.id, fight.name, minutes, seconds, boss_health)
                }
            } else {
                format!("{:2} - {} ({}:{:02})", fight.id, fight.name, minutes, seconds)
            };
            output.push(line);
        }
    }

    Ok(output)
}

#[tauri::command]
fn query_effects(state: State<'_, AppState>) -> Result<Vec<Effect>, String> {
    let stored_logs = state.logs.read().map_err(|e| e.to_string())?;
    let logs = stored_logs.as_ref().ok_or("No logs loaded")?;

    let mut effects = Vec::new();
    for log in logs {
        for (_, effect) in &log.effects {
            effects.push(effect.clone());
        }
    }

    Ok(effects)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state: State<AppState> = app.state::<AppState>();
            let default_path = app.path().resolve("Elder Scrolls Online/live/logs/Encounter.log", BaseDirectory::Document).unwrap();
            if let Err(e) = load_file(default_path, state) {
                println!("Failed to load log file on startup: {}", e);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            query_fights,
            query_effects,
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
