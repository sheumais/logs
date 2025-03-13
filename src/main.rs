mod unit;
mod fight;
mod player;
mod effect;
mod event;
mod log;
mod ui;
mod set;

use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use effect::EffectChangeType;
use crate::ui::*;

// use num_format::{Locale, ToFormattedString}; // todo make ui.rs for handling all console UI stuff with colours
use crate::log::Log;

fn read_file(file_path: &str) -> io::Result<Vec<Log>> {
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

    if query == "colours" {
        print_colour_test();
    } else {
        let logs = read_file(file_path).unwrap();
        let query_id: u32 = query.parse::<u32>().unwrap_or(0);
        let mut effect_name = "Unknown".to_string();
        let log_analysis = &logs[0];

        if query_id != 0 {
            for (_index, effect) in &log_analysis.effects {
                if effect.id == query_id {
                    effect_name = effect.name.clone();
                }
            }
    
            for (_index, fight) in &log_analysis.fights {
                let mut time_with_buff = 0;
                let mut gained_buff_timestamp = 0;
                let mut has_buff: bool = false;
                for player in &fight.players {
                    if player.unit_id == 1 {
                        if player.effects.contains(&query_id) {
                            has_buff = true;
                            gained_buff_timestamp = fight.start_time;
                        }
                    }
                }
                for effect_event in &fight.effect_events {
                    if effect_event.target_unit_state.unit_id == 1 && effect_event.ability_id == query_id {
                        if effect_event.change_type == EffectChangeType::Gained && !has_buff {
                            gained_buff_timestamp = effect_event.time;
                            has_buff = true;
                        } else if effect_event.change_type == EffectChangeType::Faded && has_buff {
                            let time_difference = effect_event.time - gained_buff_timestamp;
                            time_with_buff += time_difference;
                            has_buff = false;
                        }
                    }
                }
                if has_buff {
                    let time_difference = fight.end_time - gained_buff_timestamp;
                    time_with_buff += time_difference;
                }
    
                let fight_duration = fight.end_time - fight.start_time;
                let percentage: f32 = if fight_duration > 0 {
                    (100 * time_with_buff / fight_duration) as f32
                } else {
                    0.0
                };
    
                println!("{}: Buff: {}, Time: {}s ({}%)", fight.name, effect_name, time_with_buff / 1000, percentage);
            }
        } else {
            for log in logs {
                for (_index, fight) in log.fights {
                    for player in &fight.players {
                        if player.gear != crate::player::empty_loadout() {
                            println!("-------------------");
                            let name = foreground_rgb(&player.display_name, Colour::from_class_id(player.class_id));
                            println!("{}\n{}", name, player.gear);
                        }
                    }
                }
                for (_, player) in log.players {
                    if player.gear != crate::player::empty_loadout() {
                        println!("-------------------");
                        let name = foreground_rgb(&player.display_name, Colour::from_class_id(player.class_id));
                        println!("{}\n{}", name, player.gear);
                    }
                }
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