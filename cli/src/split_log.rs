use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::path::{Path, PathBuf};
use std::error::Error;

pub fn split_encounter_file_into_log_files(file_path: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut current_writer: Option<File> = None;

    for line in reader.lines() {
        let line = line?;
        let mut parts = line.splitn(4, ',');
        let _ = parts.next();
        let linetype = parts.next();
        let timestamp = parts.next();

        if let (Some("BEGIN_LOG"), Some(time)) = (linetype, timestamp) {
            let out_name = format!("Split-encounter-{}.log", time);
            let out_path = file_path.parent().unwrap_or_else(|| Path::new(".")).join(out_name);
            current_writer = Some(File::create(out_path)?);
        }

        if let Some(writer) = current_writer.as_mut() {
            writeln!(writer, "{line}")?;
        }
    }

    Ok(())
}

pub fn combine_encounter_log_files(file_paths: &[PathBuf]) -> Result<(), Box<dyn Error>> {
    if file_paths.is_empty() {
        return Err("No files provided".into());
    }

    let first_file = File::open(&file_paths[0])?;
    let mut first_reader = BufReader::new(first_file);
    let mut first_line = String::new();
    first_reader.read_line(&mut first_line)?;
    let start_timestamp = first_line.splitn(4, ',').nth(2)
        .ok_or("Malformed BEGIN_LOG line in first file")?
        .trim();

    let last_file = File::open(file_paths.last().unwrap())?;
    let mut last_reader = BufReader::new(last_file);
    let mut last_line = String::new();
    last_reader.read_line(&mut last_line)?;
    let end_timestamp = last_line.splitn(4, ',').nth(2)
        .ok_or("Malformed BEGIN_LOG line in last file")?
        .trim();

    let out_name = format!("Combined-encounter-{}-{}.log", start_timestamp, end_timestamp);
    let out_path = Path::new(&out_name);
    let mut out_file = File::create(out_path)?;

    for path in file_paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            writeln!(out_file, "{line}")?;
        }
    }

    Ok(())
}