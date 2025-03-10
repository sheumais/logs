mod unit;
mod fight;
mod player;
mod effect;
mod event;
mod log;

use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
// use num_format::{Locale, ToFormattedString};
use crate::log::Log;

fn read_file(file_path: &str) -> io::Result<Vec<Log>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines().peekable();

    let mut logs = Vec::new();
    let mut current_log = Log::new();

    while let Some(line) = lines.next() {
        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        let second_value = parts[1];

        match second_value {
            "BEGIN_LOG" => {
                if !current_log.is_empty() {
                    logs.push(current_log);
                }
                current_log = Log::new();
                current_log.parse_line(&line);
            }
            "END_LOG" => {
                logs.push(current_log);
                current_log = Log::new();
            }
            _ => {
                current_log.parse_line(&line);
            }
        }
    }

    if !current_log.is_empty() {
        logs.push(current_log);
    }

    Ok(logs)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (file_path, query) = parse_config(&args);
    let logs = read_file(file_path).unwrap();

    for log_analysis in logs {
        println!("{}", log_analysis.log_epoch);
    }
}

fn parse_config(args: &[String]) -> (&str, &str) {
    let mut query = "";
    let file_path = &args[1];
    if args.len() > 2 {
        query = &args[2];
    }

    (file_path, query)
}