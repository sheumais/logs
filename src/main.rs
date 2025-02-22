mod unit;
mod fight;
mod player;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};


struct Log {
    log_epoch: i64,
    players: HashMap<i32, player::Player>,
    units: HashMap<i32, unit::Unit>,
    fights: HashMap<i16, fight::Fight>,
    effects: HashMap<i32, unit::Effect>,
}

impl Log {
    fn new() -> Self {
        Self {
            log_epoch: 0,
            players: HashMap::new(),
            units: HashMap::new(),
            fights: HashMap::new(),
            effects: HashMap::new(),
        }
    }

    fn read_file(&mut self, file_path: &str) -> io::Result<()> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            self.parse_line(&line.unwrap());
        }

        Ok(())
    }

    fn parse_line(&mut self, line: &str) {
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
    
        if parts.len() >= 2 {
            match parts[1] {
                "BEGIN_LOG" => self.handle_begin_log(parts),
                "BEGIN_COMBAT" | "END_COMBAT" => self.handle_combat_change(parts),
                "UNIT_ADDED" => self.handle_unit_added(parts),
                // "PLAYER_INFO" => self.handle_player_info(parts),
                "PLAYER_INFO" => {println!("{:?}", parts)},
                _ => {},
            }
        }
    }
    
    fn get_current_fight(&mut self) -> &mut fight::Fight {
        self.fights.get_mut(&(self.fights.len() as i16 - 1)).unwrap()
    }

    fn is_true(value: &str) -> bool {
        value == "T"
    }

    fn handle_begin_log(&mut self, parts: Vec<&str>) {
        self.log_epoch = parts[2].parse::<i64>().unwrap();
        let log_version = parts[3];
        if log_version != "15" {
            panic!("Unknown log version: {} (Expected 15)", log_version);
        }
    }

    fn handle_unit_added(&mut self, parts: Vec<&str>) {
        let unit_id: i32 = parts[2].parse::<i32>().unwrap();
        if parts[3] == "PLAYER" {
            let display_name = parts[11].to_string();
            let name = parts[10].to_string();
            let player_per_session_id: i32 = parts[5].parse::<i32>().unwrap();
            if display_name != "\"\"" {
                let player = player::Player {
                    unit_id: unit_id,
                    unit_type: unit::UnitType::Player,
                    is_local_player: Self::is_true(parts[4]),
                    player_per_session_id: player_per_session_id,
                    class_id: player::match_class(parts[8]),
                    race_id: player::match_race(parts[9]),
                    name: name,
                    display_name: display_name,
                    character_id: parts[12].parse::<i128>().unwrap(),
                    level: parts[13].parse::<i8>().unwrap(),
                    champion_points: parts[14].parse::<i16>().unwrap(),
                    is_grouped_with_local_player: Self::is_true(parts[17]),
                    unit_state: unit::blank_unit_state(),
                    effects: Vec::new(),
                };
                self.players.insert(unit_id, player);
            }
        } else if parts[3] == "MONSTER" {
            let monster = unit::Unit {
                unit_id: unit_id,
                unit_type: unit::UnitType::Monster,
                monster_id: parts[6].parse::<i32>().unwrap(),
                is_boss: Self::is_true(parts[7]),
                name: parts[10].to_string(),
                level: parts[13].parse::<i8>().unwrap(),
                champion_points: parts[14].parse::<i16>().unwrap(),
                owner_unit_id: parts[15].parse::<i32>().unwrap(),
                reaction: unit::match_reaction(parts[16]),
                unit_state: unit::blank_unit_state(),
            };
            self.units.insert(unit_id, monster);
        }
    }

    fn handle_combat_change(&mut self, parts: Vec<&str>) {
        if parts[1] == "BEGIN_COMBAT" {
            self.fights.insert(self.fights.len() as i16, fight::Fight {
                id: self.fights.len() as i16,
                players: Vec::new(),
                monsters: Vec::new(),
                bosses: Vec::new(),
                start_time: parts[0].parse::<i64>().unwrap(),
                end_time: 0,
            });
        } else if parts[1] == "END_COMBAT" {
            self.get_current_fight().end_time = parts[0].parse::<i64>().unwrap();
        }
    }

    // unitId, [longTermEffectAbilityId,...], [longTermEffectStackCounts,...], [<equipmentInfo>,...], [primaryAbilityId,...], [backupAbilityId,...]
    // ["934981", "PLAYER_INFO", "47", "142210,142079,78219,89959,89958,89957,150054,172646,193447,147226,13975,184887,184873,45557,45549,63601,58955,45562,142092,184858,63880,142218,45565,185239,45601,184923,45603,45607,29062,40393,45596,39248,184847,55676,184932,185058,63802,55386,36588,185186,185243,183049,45500,15594,45267,45270,45272,185195,45514,186233,99875,45513,45509,186230,186231,186232", "1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1", "HEAD,94779,T,16,ARMOR_DIVINES,LEGENDARY,270,STAMINA,T,16,LEGENDARY", "NECK,194512,T,16,JEWELRY_BLOODTHIRSTY,LEGENDARY,694,INCREASE_PHYSICAL_DAMAGE,T,16,LEGENDARY", "CHEST,187412,T,16,ARMOR_DIVINES,LEGENDARY,652,STAMINA,T,16,LEGENDARY", "SHOULDERS,147377,T,16,ARMOR_DIVINES,LEGENDARY,127,STAMINA,T,16,LEGENDARY", "MAIN_HAND,87874,T,16,WEAPON_NIRNHONED,LEGENDARY,127,FIERY_WEAPON,T,16,LEGENDARY", "OFF_HAND,87874,T,16,WEAPON_CHARGED,LEGENDARY,127,POISONED_WEAPON,T,16,LEGENDARY", "WAIST,187472,T,16,ARMOR_DIVINES,LEGENDARY,652,STAMINA,T,16,LEGENDARY", "LEGS,187452,T,16,ARMOR_DIVINES,LEGENDARY,652,STAMINA,T,16,LEGENDARY", "FEET,187423,T,16,ARMOR_DIVINES,LEGENDARY,652,STAMINA,T,16,LEGENDARY", "RING1,147512,T,16,JEWELRY_BLOODTHIRSTY,LEGENDARY,127,INCREASE_PHYSICAL_DAMAGE,T,16,LEGENDARY", "RING2,147512,T,16,JEWELRY_BLOODTHIRSTY,LEGENDARY,127,INCREASE_PHYSICAL_DAMAGE,T,16,LEGENDARY", "HAND,187432,T,16,ARMOR_DIVINES,LEGENDARY,652,STAMINA,T,16,LEGENDARY", "BACKUP_MAIN,166198,T,16,WEAPON_INFUSED,LEGENDARY,526,BERSERKER,T,16,LEGENDARY", "40382,40195,38901,183006,186366,40161", "39011,186229,183241,182988,185842,189867"]
    fn handle_player_info(&mut self, parts: Vec<&str>) {
        let unit_id: i32 = parts[2].parse::<i32>().unwrap();
        let effect_id_list: Vec<i32> = parts[3].split(",").map(|x| x.parse::<i32>().unwrap()).collect();
        let effect_stacks_list: Vec<i16> = parts[4].split(",").map(|x| x.parse::<i16>().unwrap()).collect();
        for (i, effect_id) in effect_id_list.iter().enumerate() {
            if let Some(player) = self.players.get_mut(&unit_id) {
                // println!("Adding effect {} to player {} who is {}", effect_id, unit_id, player.display_name);
                player.effects.push(*effect_id);
            }
        }
    }

    // fn handle_equipment_info(&mut self, part: &str) -> player::GearPiece {
    //     // let gear_piece = player::GearPiece {};
    //     // gear_piece
    // }

    fn query_fights(&self) {
        for fight in self.fights.values() {
            println!("{}", fight);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (file_path, query) = parse_config(&args);

    let mut analyser = Log::new();
    analyser.read_file(file_path).unwrap();
    analyser.query_fights();
}

fn parse_config(args: &[String]) -> (&str, &str) {
    let mut query = "";
    let file_path = &args[1];
    if args.len() > 2 {
        query = &args[2];
    }

    (file_path, query)
}