use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::Path;
use parser::ui::*;
use parser::log::Log;

use crate::log_edit::modify_log_file;
pub mod log_edit;

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

fn main() {
    let args: Vec<String> = env::args().collect();
    let (file_path, query) = parse_config(&args);

    match query {
        "colours" => {
            print_colour_test();
        }
        "modify" => {
            if let Err(e) = modify_log_file(Path::new(file_path)) {
                eprintln!("Error modifying log file: {}", e);
            }
        }
        "fights" => {
            let logs = read_file(Path::new(file_path)).unwrap();
            for log in logs {
                for fight in log.fights {
                    let duration_secs = (fight.end_time - fight.start_time) / 1000;
                    let minutes = duration_secs / 60;
                    let seconds = duration_secs % 60;
                    let boss_health_opt = fight.get_average_boss_health_percentage();
                    if let Some(boss_health) = boss_health_opt {
                        if boss_health == 0.0 {
                            println!("{:2} - {} ({}:{:02}) KILL", fight.id, fight.name, minutes, seconds);
                        } else {
                            println!("{:2} - {} ({}:{:02}) {:.0}%", fight.id, fight.name, minutes, seconds, boss_health);
                        }
                    } else {
                        println!("{:2} - {} ({}:{:02})", fight.id, fight.name, minutes, seconds);
                    }
                }
            }
        }
        "gear" => {
            let logs = read_file(Path::new(file_path)).unwrap();
            for log in logs {
                for fight in log.fights {
                    for player in &fight.players {
                        if player.gear != parser::player::empty_loadout() {
                            println!("-------------------");
                            let name = foreground_rgb(&player.display_name, Colour::from_class_id(player.class_id));
                            println!("{}\n{}", name, player.gear);
                            for skill_set in [&player.primary_abilities, &player.backup_abilities] {
                                for ability in skill_set {
                                    println!("{:?}", ability);
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {
            let logs = read_file(Path::new(file_path)).unwrap();
            let query_id: u32 = query.parse::<u32>().unwrap_or(0);
            let mut effect_name = "Unknown".to_string();
            let log_analysis = &logs[0];

            if query_id != 0 {
                for (_index, effect) in &log_analysis.abilities {
                    if effect.id == query_id {
                        effect_name = effect.name.clone();
                    }
                }
                
                println!("Uptime of {}", effect_name);
                for fight in &log_analysis.fights {
                    let uptime = parser::effect::buff_uptime_over_fight(query_id, 1, fight);
                    println!("{:.2}%", 100.0 * uptime);
                };
            }
        }
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