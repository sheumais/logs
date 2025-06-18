use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::path::Path;
use parser::log::Log;
use parser::player::GearSlot;
use parser::set::{get_item_type_from_hashmap, ItemType};

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
            if parts_clone.get(1) != Some(&"ABILITY_INFO") && parts_clone.get(1) != Some(&"EFFECT_INFO")  && parts_clone.get(1) != Some(&"PLAYER_INFO") {
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
        "ABILITY_INFO" => check_ability_info(parts),
        "EFFECT_INFO" => add_arcanist_beam_effect_information(parts),
        "PLAYER_INFO" => modify_player_data(parts),
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

fn check_ability_info(parts: Vec<&str>) -> Option<Vec<String>> {
    if parts[2] == PRAGMATIC || parts[2] == EXHAUSTING {
        add_arcanist_beam_information(parts)
    } else if parts[2].parse::<u32>().ok() == Some(BLOCKADE_DEFAULT) {
        add_blockade_versions(parts)     
    } else {
        return None
    }
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

const BLOCKADE_FIRE: u32 = 39012;
const BLOCKADE_STORMS: u32 = 39018;
const BLOCKADE_FROST: u32 = 39028;
const BLOCKADE_DEFAULT: u32 = 39011;

fn add_blockade_versions(parts: Vec<&str>) -> Option<Vec<String>> {
    let mut lines = Vec::new();
    // ABILITY_INFO,39011,"Elemental Blockade","/esoui/art/icons/ability_destructionstaff_002a.dds",T,T
    // ABILITY_INFO,39028,"Blockade of Frost","/esoui/art/icons/ability_destructionstaff_002b.dds",F,T
	// ABILITY_INFO,39012,"Blockade of Fire","/esoui/art/icons/ability_destructionstaff_004_b.dds",F,T
    // ABILITY_INFO,39018,"Blockade of Storms","/esoui/art/icons/ability_destructionstaff_003_b.dds",F,T
	// ABILITY_INFO,62951,"Blockade of Frost","/esoui/art/icons/ability_destructionstaff_002b.dds",F,F
	// ABILITY_INFO,62912,"Blockade of Fire","/esoui/art/icons/ability_destructionstaff_004_b.dds",F,F
	// ABILITY_INFO,62990,"Blockade of Storms","/esoui/art/icons/ability_destructionstaff_003_b.dds",F,F
    lines.push(format!("{},{},{},\"{}\",\"{}\",{},{}", parts[0], "ABILITY_INFO", BLOCKADE_FIRE, "Blockade of Fire", "/esoui/art/icons/ability_destructionstaff_004_b.dds", "F", "T"));
    lines.push(format!("{},{},{},\"{}\",\"{}\",{},{}", parts[0], "ABILITY_INFO", BLOCKADE_STORMS, "Blockade of Storms", "/esoui/art/icons/ability_destructionstaff_003_b.dds", "F", "T"));
    lines.push(format!("{},{},{},\"{}\",\"{}\",{},{}", parts[0], "ABILITY_INFO", BLOCKADE_FROST, "Blockade of Frost", "/esoui/art/icons/ability_destructionstaff_002b.dds", "F", "T"));
    return Some(lines);
}

fn modify_player_data(parts: Vec<&str>) -> Option<Vec<String>> {
    
    if parts.len() < 7 { // this can occur if either the player is wearing nothing and has no skills, or they're not in the trial.
        return None;
    }

    let mut primary_ability_id_list: Vec<u32> = parts[parts.len() - 2].split(',').map(|x| x.parse::<u32>().unwrap_or_default()).collect();
    let mut backup_ability_id_list: Vec<u32> = parts[parts.len() - 1].split(',').map(|x| x.parse::<u32>().unwrap_or_default()).collect();

    let gear_parts = parts.len() - 2;
    let mut frontbar_type = ItemType::Unknown;
    let mut backbar_type = ItemType::Unknown;
    for i in 5..gear_parts {
        let gear_piece = Log::handle_equipment_info(parts[i]);
        let item_slot = gear_piece.slot;
        let item_type = get_item_type_from_hashmap(gear_piece.item_id);
        if item_slot == GearSlot::MainHand {
            frontbar_type = item_type;
        } else if item_slot == GearSlot::MainHandBackup {
            backbar_type = item_type;
        }
    }

    for id in &mut primary_ability_id_list {
        if *id == BLOCKADE_DEFAULT || *id == BLOCKADE_FIRE || *id == BLOCKADE_FROST || *id == BLOCKADE_STORMS {
            *id = match frontbar_type {
                ItemType::FrostStaff => BLOCKADE_FROST,
                ItemType::FireStaff => BLOCKADE_FIRE,
                ItemType::LightningStaff => BLOCKADE_STORMS,
                _ => BLOCKADE_DEFAULT,
            };
        }
    }

    for id in &mut backup_ability_id_list {
        if *id == BLOCKADE_DEFAULT || *id == BLOCKADE_FIRE || *id == BLOCKADE_FROST || *id == BLOCKADE_STORMS {
            *id = match backbar_type {
                ItemType::FrostStaff => BLOCKADE_FROST,
                ItemType::FireStaff => BLOCKADE_FIRE,
                ItemType::LightningStaff => BLOCKADE_STORMS,
                _ => BLOCKADE_DEFAULT,
            };
        }
    }
    
    let mut new_parts: Vec<String> = vec![
        format!("{}", parts[0]),
        format!("{}", parts[1]),
        format!("{}", parts[2]),
        format!("[{}]", parts[3]),
        format!("[{}]", parts[4]),
    ];
    let gear_start = 5;
    let gear_end = parts.len().saturating_sub(2);
    let gear: Vec<String> = if parts.len() > gear_start && gear_end > gear_start {
        parts[gear_start..gear_end]
            .iter()
            .map(|p| format!("[{}]", p))
            .collect()
    } else {
        vec![]
    };
    new_parts.push(format!("[{}]", gear.join(",")));
    new_parts.push(format!("[{}]", primary_ability_id_list.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")));
    new_parts.push(format!("[{}]", backup_ability_id_list.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")));


    Some(vec![new_parts.join(",")])
}