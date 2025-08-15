use std::{fs::File, io::{self, BufRead, BufReader}, path::Path};
use parser::log::Log;
pub mod log_edit;
pub mod split_log;
pub mod esologs_format;
pub mod esologs_convert;

pub fn read_file(file_path: &Path) -> io::Result<Vec<Log>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines().peekable();

    let mut logs = Vec::new();
    let mut current_log = Log::new();

    while let Some(line) = lines.next() {
        let line = line?;
        let mut in_brackets = false;
        let mut in_quotes = false;
        let mut start = 0;
        let mut just_closed_quote = false; 
        let mut parts: Vec<&str> = Vec::new();

        let mut iter = line.char_indices().peekable();
        while let Some((i, ch)) = iter.next() {
            match ch {
                '[' if !in_quotes => { in_brackets = true;  start = i + 1; }
                ']' if !in_quotes => {
                    in_brackets = false;
                    parts.push(&line[start..i]);
                    start = i + 1;
                }

                '"' => {
                    if in_quotes && iter.peek().map(|(_,c)| *c) == Some('"') {
                        iter.next();
                        continue;
                    }

                    if in_quotes {
                        parts.push(&line[start..i]);
                        in_quotes = false;
                        just_closed_quote = true;
                        start = i + 1;
                    } else {
                        in_quotes = true;
                        start = i + 1;
                    }
                }

                ',' if !in_brackets && !in_quotes => {
                    if just_closed_quote {
                        just_closed_quote = false;
                        start = i + 1;
                    } else {
                        parts.push(&line[start..i]);
                        start = i + 1;
                    }
                }

                _ => {}
            }
        }

        if start < line.len() || just_closed_quote {
            parts.push(&line[start..]);
        }
        let second_value = parts[1];

        match second_value {
            "BEGIN_LOG" => {
                if !current_log.fights.is_empty() {
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

    if !current_log.fights.is_empty() {
        logs.push(current_log);
    }

    Ok(logs)
}