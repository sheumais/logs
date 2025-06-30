use std::collections::HashMap;
use std::env;
use std::path::Path;
use cli::esologs_convert::ESOLogProcessor;
use cli::log_edit::modify_log_file;
use cli::read_file;
use cli::split_log::split_encounter_file_into_log_files;
use parser::ui::*;

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
        "split" => {
            if let Err(e) = split_encounter_file_into_log_files(Path::new(file_path)) {
                eprintln!("Error splitting log file: {}", e);
            }
        }
        "esolog" => {
            let mut eso_log_processor = ESOLogProcessor::new();
            if let Err(e) = eso_log_processor.convert_log_file_to_esolog_format(Path::new(file_path)) {
                eprintln!("Error splitting log file: {}", e);
            }
            use std::fs::File;
            use std::io::Write;
            let mut file = match File::create("C:/Users/H/Downloads/esolog_output.txt") {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error creating output file: {}", e);
                    return;
                }
            };
            println!("{}", eso_log_processor.eso_logs_log.units.len());
            for unit in eso_log_processor.eso_logs_log.units {
                if let Err(e) = writeln!(file, "{unit}") {
                    eprintln!("Error writing to file: {}", e);
                }
            }
            { // Fix stuff for some buffs
                let default_icon = "ability_mage_065";
                let mut icon_by_name = HashMap::<String, String>::new();
                for buff in eso_log_processor.eso_logs_log.buffs.iter() {
                    if buff.icon != default_icon {
                        icon_by_name.insert(buff.name.clone(), buff.icon.clone());
                    }
                }
                println!("{}", eso_log_processor.eso_logs_log.buffs.len());
                for buff in eso_log_processor.eso_logs_log.buffs.iter_mut() {
                    if buff.icon == default_icon {
                        if let Some(icon) = icon_by_name.get(&buff.name) {
                            buff.icon = icon.clone();
                        }
                    }
                    if let Err(e) = writeln!(file, "{buff}") {
                        eprintln!("Error writing to file: {e}");
                    }
                }
            }
            println!("{}", eso_log_processor.eso_logs_log.effects.len());
            for event in eso_log_processor.eso_logs_log.effects {
                if let Err(e) = writeln!(file, "{event}") {
                    eprintln!("Error writing to file: {}", e);
                }
            }
            let mut file = match File::create("C:/Users/H/Downloads/esolog_output2.txt") {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error creating output file: {}", e);
                    return;
                }
            };
            for line in eso_log_processor.eso_logs_log.events.iter() {
                if let Err(e) = writeln!(file, "{line}") {
                    eprintln!("Error writing to file: {}", e);
                }
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
            let log_analysis = &logs[2];

            if query_id != 0 {
                for (_index, effect) in &log_analysis.abilities {
                    if effect.id == query_id {
                        effect_name = effect.name.clone();
                    }
                }
                
                println!("Uptime of {}", effect_name);
                for fight in &log_analysis.fights {
                    for player in fight.players.iter() {
                        let uptime = parser::effect::buff_uptime_over_fight(query_id, player.unit_id, fight);
                        println!("{} {} {:.2}%", fight.name, player.display_name, 100.0 * uptime);
                    }
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