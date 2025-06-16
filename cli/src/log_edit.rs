use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::path::Path;

pub fn modify_log_file(file_path: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut modified_lines: Vec<String> = Vec::new();

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                continue;
            }
        };
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
            modified_lines.extend(new_lines);
            if parts_clone.get(1) != Some(&"ABILITY_INFO") && parts_clone.get(1) != Some(&"EFFECT_INFO") {
                modified_lines.push(line.clone());
            }
        } else {
            modified_lines.push(line.clone());
        }
    }

    let mut new_path = file_path.with_extension("");
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
        .open(&new_path)?;
    let mut writer = BufWriter::new(file);

    for line in &modified_lines {
        writeln!(writer, "{}", line)
            .map_err(|e| format!("Failed to write line: {}", e))?;
    }
    Ok(())
}

pub fn check_line_for_edits(parts: Vec<&str>) -> Option<Vec<String>> {
    match parts[1] {
        "EFFECT_CHANGED" => add_arcanist_beam_cast(parts),
        "ABILITY_INFO" => add_arcanist_beam_information(parts),
        "EFFECT_INFO" => add_arcanist_beam_effect_information(parts),
        _ => None,
    }
}

const PRAGMATIC: &str = "186369";
const EXHAUSTING: &str = "186780";

fn add_arcanist_beam_cast(parts: Vec<&str>) -> Option<Vec<String>> {
    if parts.len() < 17 {
        return None;
    }
    if parts[5] == PRAGMATIC || parts[5] == EXHAUSTING {
        if parts[2] == "GAINED" {
            let duration = 4500 + if parts[5] == EXHAUSTING { 1000 } else { 0 };
            let mut lines = Vec::new();
            let mut line = format!("{},{},{},{},", parts[0], "BEGIN_CAST", 0, "F");
            let rest = parts[4..].join(",");
            line.push_str(&rest);
            lines.push(line);
            line = format!("{},{},{},{},", parts[0], "BEGIN_CAST", duration, "T");
            line.push_str(&rest);
            lines.push(line);
            return Some(lines);
        } else if parts[2] == "FADED" {
            return Some(vec![format!("{},{},{},{},{}", parts[0], "END_CAST", "COMPLETED", parts[4], parts[5])]);
        }
    }
    return None;
}

fn add_arcanist_beam_information(parts: Vec<&str>) -> Option<Vec<String>> {
    let mut lines = Vec::new();
    if parts[2] == PRAGMATIC {
        lines.push(format!("{},{},{},{},{},{},{}", parts[0], parts[1], PRAGMATIC, parts[3], "\"/esoui/art/icons/ability_arcanist_002_b.dds\"", "F", "T"));
        return Some(lines);
    } else if parts[2] == EXHAUSTING {
        lines.push(format!("{},{},{},{},{},{},{}", parts[0], parts[1], EXHAUSTING, parts[3], "\"/esoui/art/icons/ability_arcanist_002_a.dds\"", "F", "T"));
        return Some(lines);
    }
    return None;
}

fn add_arcanist_beam_effect_information(parts: Vec<&str>) -> Option<Vec<String>> {
    let mut lines = Vec::new();
    if parts[2] == PRAGMATIC {
        lines.push(format!("{},{},{},{},{},{}", parts[0], "EFFECT_INFO", PRAGMATIC, "BUFF", "NONE", "NEVER"));
        return Some(lines);
    } else if parts[2] == EXHAUSTING {
        lines.push(format!("{},{},{},{},{},{}", parts[0], "EFFECT_INFO", EXHAUSTING, "BUFF", "NONE", "NEVER"));
        return Some(lines);
    }
    return None;
}