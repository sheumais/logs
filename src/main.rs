mod unit;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use unit::{Unit, UnitType, Reaction, is_true, match_class, match_race, match_reaction};


fn main() {
    let args: Vec<String> = env::args().collect();

    let (file_path, query) = parse_config(&args);

    let mut units: HashMap<i32, unit::Unit> = HashMap::new();
    
    read_file(file_path, &mut units).unwrap();

    if query == "all" {
        for unit in units.values() {
            println!("{}", unit);
        }
    } else if query == "count" {
        println!("Number of units: {}", units.len());
    } else if query == "monster" {
        for unit in units.values() {
            if unit.unit_type == unit::UnitType::Monster {
                println!("{}", unit);
            }
        }
    } else if query == "boss" {
        for unit in units.values() {
            if unit.is_boss {
                println!("{}", unit);
            }
        }
    } else if query == "player" {
        for unit in units.values() {
            if unit.unit_type == unit::UnitType::Player {
                println!("{}", unit);
            }
        }
    } else if query == "hostile" {
        for unit in units.values() {
            if unit.reaction == unit::Reaction::Hostile {
                println!("{}", unit);
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

fn read_file(file_path: &str, units: &mut HashMap<i32, unit::Unit>) -> io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        parse_line(&line.unwrap(), units);
    }

    Ok(())
}

fn parse_line(line: &str, units: &mut HashMap<i32, unit::Unit>) {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 2 {
        let second_value = parts[1];
        match second_value {
            "EFFECT_CHANGED" => {}
            "ABILITY_INFO" => {}
            "EFFECT_INFO" => {}
            "UNIT_ADDED" => {handle_unit_added(parts, units);}
            "UNIT_REMOVED" => {}
            "COMBAT_EVENT" => {}
            "BEGIN_CAST" => {}
            "END_CAST" => {}
            "UNIT_CHANGED" => {}
            "HEALTH_REGEN" => {}
            "PLAYER_INFO" => {}
            "BEGIN_COMBAT" => {}
            "END_COMBAT" => {}
            "MAP_CHANGED" => {}
            "BEGIN_TRIAL" => {}
            "TRIAL_INIT" => {}
            "END_TRIAL" => {}
            "ZONE_CHANGED" => {}
            "BEGIN_LOG" => {}
            "END_LOG" => {}
            _ => {
                println!("Unknown event: {}", second_value); 
            }
        }
    }
}

fn handle_unit_added(parts: Vec<&str>, units: &mut HashMap<i32, unit::Unit>) {
    let unit_id: i32 = parts[2].parse::<i32>().unwrap();
    if parts[3] == "PLAYER" {
        units.insert(unit_id, unit::Unit {
            unit_id: unit_id,
            unit_type: unit::UnitType::Player,
            is_local_player: unit::is_true(parts[4]),
            player_per_session_id: parts[5].parse::<i32>().unwrap(),
            monster_id: 0,
            is_boss: false,
            class_id: unit::match_class(parts[8]),
            race_id: unit::match_race(parts[9]),
            name: parts[10].to_string(),
            display_name: parts[11].to_string(),
            character_id: parts[12].parse::<i128>().unwrap(),
            level: parts[13].parse::<i8>().unwrap(),
            champion_points: parts[14].parse::<i16>().unwrap(),
            owner_unit_id: parts[15].parse::<i32>().unwrap(),
            reaction: unit::match_reaction(parts[16]),
            is_grouped_with_local_player: unit::is_true(parts[17]),
        });
    } else if parts[3] == "MONSTER" {
        units.insert(unit_id, unit::Unit {
            unit_id: unit_id,
            unit_type: unit::UnitType::Monster,
            is_local_player: false,
            player_per_session_id: 0,
            monster_id: parts[6].parse::<i32>().unwrap(),
            is_boss: unit::is_true(parts[7]),
            class_id: unit::ClassId::None,
            race_id: unit::RaceId::None,
            name: parts[10].to_string(),
            display_name: "".to_owned(),
            character_id: 0,
            level: parts[13].parse::<i8>().unwrap(),
            champion_points: parts[14].parse::<i16>().unwrap(),
            owner_unit_id: parts[15].parse::<i32>().unwrap(),
            reaction: unit::match_reaction(parts[16]),
            is_grouped_with_local_player: false,
        });
    }
}
