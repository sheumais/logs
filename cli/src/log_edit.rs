use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::path::Path;
use parser::effect::{is_zen_dot, MOULDERING_TAINT_ID, MOULDERING_TAINT_TIME, ZEN_DEBUFF_ID};
use parser::event::{self, EventResult};
use parser::parse::{self, gear_piece, unit_state};
use parser::player::GearSlot;
use parser::set::{get_item_type_from_hashmap, ItemType};
use parser::unit::UnitState;
use parser::{EffectChangedEventType, EventType};

pub struct CustomLogData {
    pub zen_stacks: HashMap<u32, ZenDebuffState>,
    pub scribing_abilities: Vec<ScribingAbility>,
    pub scribing_map: HashMap<u32, usize>,
    pub scribing_unit_map: HashMap<(u32, u32), usize>,
    pub taint_stacks: HashMap<u32, MoulderingTaintState>,
}

impl CustomLogData {
    pub fn new() -> Self {
        CustomLogData {
            zen_stacks: HashMap::new(),
            scribing_abilities: Vec::new(),
            scribing_map: HashMap::new(),
            scribing_unit_map: HashMap::new(),
            taint_stacks: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.zen_stacks = HashMap::new();
        self.taint_stacks = HashMap::new();
    }
}

pub struct ZenDebuffState {
    active: bool,
    source_id: u32,
    contributing_ability_ids: Vec<u32>,
}

pub struct MoulderingTaintState {
    stacks: u8,
    last_timestamp: u64,
    last_source_unit_state: UnitState,
    last_target_unit_state: UnitState,
    last_cast_id: u32,
}

const BEGIN_SCRIBING_ABILITIES: u32 = 1000;

#[derive(Debug, Clone)]
pub struct ScribingAbility {
    pub id: u32,
    pub name: String,
    pub icon: String,
    pub scribing: Option<Vec<String>>,
}

pub fn modify_log_file(file_path: &Path) -> Result<(), Box<dyn Error>> {
    let file: File = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut modified_lines: Vec<String> = Vec::new();
    let mut custom_log_data = CustomLogData::new();

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                log::warn!("Error reading line: {}", e);
                continue;
            }
        };
        modified_lines.extend(handle_line(line, &mut custom_log_data));
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

pub fn handle_line(line: String, custom_log_data: &mut CustomLogData) -> Vec<String> {
    let parts: Vec<String> = parser::parse::handle_line(&line);
    
    if parts.get(1).map(|s| s.as_str()) == Some("BEGIN_COMBAT") {
        custom_log_data.zen_stacks.clear();
    }

    let new_addition = check_line_for_edits(&parts, custom_log_data);

    let mut modified_lines = Vec::new();

    if let Some(new_lines) = new_addition {
        let is_ability = parts.get(1).map(|s| s.as_str()) == Some("ABILITY_INFO");
        let is_effect  = parts.get(1).map(|s| s.as_str()) == Some("EFFECT_INFO");
        let is_player  = parts.get(1).map(|s| s.as_str()) == Some("PLAYER_INFO");

        let is_zen_debuff = parts
            .get(5)
            .and_then(|s| s.parse::<u32>().ok())
            == Some(*ZEN_DEBUFF_ID);

        if !is_ability && !is_effect && !is_player && !is_zen_debuff {
            modified_lines.push(line);
        }

        modified_lines.extend(new_lines);
    } else {
        modified_lines.push(line);
    }

    modified_lines
}

fn check_line_for_edits(parts: &[String], custom_log_data: &mut CustomLogData) -> Option<Vec<String>> {
    let event = parts.get(1).map(|s| EventType::from(s.as_str())).unwrap_or(EventType::Unknown);
    match event {
        EventType::EffectChanged => check_effect_changed(parts, &mut custom_log_data.zen_stacks),
        EventType::AbilityInfo => check_ability_info(parts, custom_log_data),
        EventType::EffectInfo => add_arcanist_beam_effect_information(parts),
        EventType::PlayerInfo => modify_player_data(parts, custom_log_data),
        EventType::CombatEvent => modify_combat_event(parts, custom_log_data),
        _ => None,
    }
}

const PRAGMATIC: &'static u32 = &186369;
const EXHAUSTING: &'static u32 = &186780;

fn check_effect_changed(parts: &[String], zen_hashmap: &mut HashMap<u32, ZenDebuffState>) -> Option<Vec<String>> {
    if parts.len() < 17 {
        return None;
    }
    match &parts[5] {
        id if *id == PRAGMATIC.to_string() => return add_arcanist_beam_cast(parts),
        id if *id == EXHAUSTING.to_string() => return add_arcanist_beam_cast(parts),
        id if *id == ZEN_DEBUFF_ID.to_string() => return add_zen_stacks(parts, zen_hashmap),
        id if is_zen_dot(id.parse::<u32>().unwrap_or(0)) => return add_zen_stacks(parts, zen_hashmap),
        _ => return None,
    }
}

const MAX_ZEN_STACKS: u8 = 5;

fn add_zen_stacks(parts: &[String], zen_status: &mut HashMap<u32, ZenDebuffState>) -> Option<Vec<String>> {
    let source_unit_state = unit_state(&parts, 6);
    let target_unit_state = if parts[16] == "*" {
        source_unit_state.clone()
    } else {
        unit_state(&parts, 16)
    };

    let source_unit_id = source_unit_state.unit_id;
    let target_unit_id = target_unit_state.unit_id;

    let is_zen_debuff = parts[5] == ZEN_DEBUFF_ID.to_string();
    let event_type = parts.get(2).map(|s| EffectChangedEventType::from(s.as_str())).unwrap_or(EffectChangedEventType::Unknown);
    let ability_id = parts[5].parse::<u32>().unwrap_or(0);

    let entry = zen_status.entry(target_unit_id).or_insert_with(|| ZenDebuffState {
        active: false,
        source_id: source_unit_id,
        contributing_ability_ids: Vec::new(),
    });

    if is_zen_debuff {
        match event_type {
            EffectChangedEventType::Gained => {
                entry.active = true;
                entry.source_id = source_unit_id;
            }
            EffectChangedEventType::Faded => {
                if source_unit_id == entry.source_id || source_unit_id == target_unit_id {
                    entry.active = false;
                }
            }
            EffectChangedEventType::Updated => {
                if source_unit_id == entry.source_id || source_unit_id == target_unit_id {
                    entry.active = true;
                }
            }
            _ => {}
        }

        let stacks = entry.contributing_ability_ids.len().min(MAX_ZEN_STACKS as usize);
        let mut line = format!(
            "{},{},{},{},{},{},",
            parts[0], parts[1], parts[2], stacks, parts[4], ZEN_DEBUFF_ID.to_string()
        );
        let rest = parts[6..].join(",");
        line.push_str(&rest);
        return Some(vec![line]);
    } else {
        if source_unit_id == entry.source_id || source_unit_id == target_unit_id {
            match event_type {
                EffectChangedEventType::Gained => {
                    if !entry.contributing_ability_ids.contains(&ability_id) {
                        entry.contributing_ability_ids.push(ability_id);
                        if entry.active {
                            let stacks = entry.contributing_ability_ids.len().min(MAX_ZEN_STACKS as usize);
                            let mut line = format!(
                                "{},{},{},{},{},{},",
                                parts[0], parts[1], "UPDATED", stacks, parts[4], ZEN_DEBUFF_ID.to_string()
                            );
                            let rest = parts[6..].join(",");
                            line.push_str(&rest);
                            return Some(vec![line]);
                        }
                    }
                }
                EffectChangedEventType::Faded => {
                    if entry.contributing_ability_ids.contains(&ability_id) {
                        entry.contributing_ability_ids.retain(|&id| id != ability_id);
                        if entry.active {
                            let stacks = entry.contributing_ability_ids.len().min(MAX_ZEN_STACKS as usize);
                            let mut line = format!(
                                "{},{},{},{},{},{},",
                                parts[0], parts[1], "UPDATED", stacks, parts[4], ZEN_DEBUFF_ID.to_string()
                            );
                            let rest = parts[6..].join(",");
                            line.push_str(&rest);
                            return Some(vec![line]);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    None
}

fn add_arcanist_beam_cast(parts: &[String]) -> Option<Vec<String>> {
    if parts[5] == PRAGMATIC.to_string() || parts[5] == EXHAUSTING.to_string() {
        if parts[2] == "GAINED" {
            let duration = 4500 + if parts[5] == EXHAUSTING.to_string() { 1000 } else { 0 };
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

fn check_ability_info(parts: &[String], custom_log_data: &mut CustomLogData) -> Option<Vec<String>> {
    let ability = parse::ability(parts);
    if ability.scribing.is_some() {
        let ability_name_clone = ability.name.clone();
        let ability_id_clone = ability.id;
        let scribing_ability = ScribingAbility {
            id: BEGIN_SCRIBING_ABILITIES + custom_log_data.scribing_abilities.len() as u32,
            name: ability.name,
            icon: ability.icon,
            scribing: ability.scribing,
        };
        custom_log_data.scribing_abilities.push(scribing_ability);
        let index = custom_log_data.scribing_abilities.len() - 1;
        let scribing_ability = &custom_log_data.scribing_abilities[index];
        custom_log_data.scribing_map.insert(ability.id, index);
        let focus_script = &scribing_ability.scribing.as_ref().unwrap()[0];
        let signature_script = &scribing_ability.scribing.as_ref().unwrap()[1];
        let affix_script = &scribing_ability.scribing.as_ref().unwrap()[2];
        let new_name = format!("{} ({} / {})", &scribing_ability.name, signature_script, affix_script);
        return Some(vec![format!("{},{},{},\"{}\",\"{}\",{},{},\"{}\",\"{}\",\"{}\"", parts[0], "ABILITY_INFO", scribing_ability.id, new_name, scribing_ability.icon, "F", "T", focus_script, signature_script, affix_script),
        format!("{},{},{},\"{}\",\"{}\",{},{},\"{}\",\"{}\",\"{}\"", parts[0], "ABILITY_INFO", ability_id_clone, ability_name_clone, scribing_ability.icon, "F", "T", focus_script, signature_script, affix_script)]);
    } else if parts[2] == PRAGMATIC.to_string() || parts[2] == EXHAUSTING.to_string() {
        add_arcanist_beam_information(parts)
    } else if parts[2].parse::<u32>().ok() == Some(BLOCKADE_DEFAULT) {
        add_blockade_versions(parts)     
    } else {
        return None
    }
}

fn add_arcanist_beam_information(parts: &[String]) -> Option<Vec<String>> {
    let mut lines = Vec::new();
    if parts[2] == PRAGMATIC.to_string() {
        lines.push(format!("{},{},{},{},{},{},{}", parts[0], parts[1], PRAGMATIC, parts[3], "\"/esoui/art/icons/ability_arcanist_002_b.dds\"", "F", "T"));
        return Some(lines);
    } else if parts[2] == EXHAUSTING.to_string() {
        lines.push(format!("{},{},{},{},{},{},{}", parts[0], parts[1], EXHAUSTING, parts[3], "\"/esoui/art/icons/ability_arcanist_002_a.dds\"", "F", "T"));
        return Some(lines);
    }
    return None;
}

fn add_arcanist_beam_effect_information(parts: &[String]) -> Option<Vec<String>> {
    let mut lines = Vec::new();
    if parts[2] == PRAGMATIC.to_string() {
        lines.push(format!("{},{},{},{},{},{}", parts[0], "EFFECT_INFO", PRAGMATIC, "BUFF", "NONE", "NEVER"));
        return Some(lines);
    } else if parts[2] == EXHAUSTING.to_string() {
        lines.push(format!("{},{},{},{},{},{}", parts[0], "EFFECT_INFO", EXHAUSTING, "BUFF", "NONE", "NEVER"));
        return Some(lines);
    }
    return None;
}

const BLOCKADE_FIRE: u32 = 39012;
const BLOCKADE_STORMS: u32 = 39018;
const BLOCKADE_FROST: u32 = 39028;
const BLOCKADE_DEFAULT: u32 = 39011;

fn add_blockade_versions(parts: &[String]) -> Option<Vec<String>> {
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

fn modify_player_data(parts: &[String], custom_log_data: &mut CustomLogData) -> Option<Vec<String>> {
    
    log::trace!("Modifying player data: {:?}", parts);

    if parts.len() < 7 { // this can occur if either the player is wearing nothing and has no skills, or they're not in the trial.
        return None;
    }

    let mut primary_ability_id_list: Vec<u32> = parts[parts.len() - 2].split(',').map(|x| x.parse::<u32>().unwrap_or_default()).collect();
    let mut backup_ability_id_list: Vec<u32> = parts[parts.len() - 1].split(',').map(|x| x.parse::<u32>().unwrap_or_default()).collect();

    let mut frontbar_type = ItemType::Unknown;
    let mut backbar_type = ItemType::Unknown;
    let gear_parts: Vec<&str> = parts[5].trim_matches(|c| c == '[' || c == ']')
    .split("],[")
    .collect();

    for i in gear_parts{
        let gear_piece = gear_piece(i);
        let item_slot = gear_piece.slot;
        let item_type = get_item_type_from_hashmap(gear_piece.item_id);
        if item_slot == GearSlot::MainHand {
            frontbar_type = item_type;
        } else if item_slot == GearSlot::MainHandBackup {
            backbar_type = item_type;
        }
    }
    // println!("{}", custom_log_data.scribing_map.len());
    // println!("{}", custom_log_data.scribing_map.keys().map(|k| k.to_string()).collect::<Vec<_>>().join(", "));
    // println!("{}", custom_log_data.scribing_abilities.iter().map(|a| a.id.to_string()).collect::<Vec<_>>().join(", "));
    // println!("{:?}", custom_log_data.scribing_abilities);

    let player_id = parts[2].parse::<u32>().unwrap();

    for id in &mut primary_ability_id_list {
        // log::trace!("Checking id: {}", id);
        // println!("Current scribing_map: {:?}", custom_log_data.scribing_map);
        if matches!(*id, BLOCKADE_DEFAULT | BLOCKADE_FIRE | BLOCKADE_FROST | BLOCKADE_STORMS) {
            *id = match frontbar_type {
                ItemType::FrostStaff => BLOCKADE_FROST,
                ItemType::FireStaff => BLOCKADE_FIRE,
                ItemType::LightningStaff => BLOCKADE_STORMS,
                _ => BLOCKADE_DEFAULT,
            };
        } else if let Some(index) = custom_log_data.scribing_unit_map.get(&(player_id, *id)) {
            log::trace!("Setting {} to originally existing index {} for {}", player_id, index, id);
            *id = BEGIN_SCRIBING_ABILITIES + *index as u32;
        } else if custom_log_data.scribing_map.contains_key(id) {
            if let Some(index) = custom_log_data.scribing_map.get(id) {
                custom_log_data.scribing_unit_map.insert((player_id, *id), *index);
                *id = BEGIN_SCRIBING_ABILITIES + *index as u32;
            }
        }
    }

    for id in &mut backup_ability_id_list {
        // log::trace!("Checking id: {}", id);
        if matches!(*id, BLOCKADE_DEFAULT | BLOCKADE_FIRE | BLOCKADE_FROST | BLOCKADE_STORMS) {
            *id = match backbar_type {
                ItemType::FrostStaff => BLOCKADE_FROST,
                ItemType::FireStaff => BLOCKADE_FIRE,
                ItemType::LightningStaff => BLOCKADE_STORMS,
                _ => BLOCKADE_DEFAULT,
            };
        } else if let Some(index) = custom_log_data.scribing_unit_map.get(&(player_id, *id)) {
            log::trace!("Setting {} to originally existing index {} for {}", player_id, index, id);
            *id = BEGIN_SCRIBING_ABILITIES + *index as u32;
        } else if custom_log_data.scribing_map.contains_key(id) {
            if let Some(index) = custom_log_data.scribing_map.get(id) {
                custom_log_data.scribing_unit_map.insert((player_id, *id), *index);
                *id = BEGIN_SCRIBING_ABILITIES + *index as u32;
            }
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

fn modify_combat_event(parts: &[String], custom_log_data: &mut CustomLogData) -> Option<Vec<String>> {
    let ability_id = parts[8].parse::<u32>().unwrap();
    log::trace!("ability_id: {}", ability_id);
    let time = parts[0].parse().unwrap();
    if ability_id == *MOULDERING_TAINT_ID {
        let source = parse::unit_state(&parts, 9);
        let target = parse::unit_state(&parts, 19);
        let cast_track_id = parts[7].parse::<u32>().unwrap(); 
        let entry = custom_log_data.taint_stacks.entry(target.unit_id).or_insert_with(|| MoulderingTaintState {
            stacks: 0,
            last_timestamp: time,
            last_source_unit_state: source,
            last_cast_id: cast_track_id,
            last_target_unit_state: target,
        });
        entry.stacks += 1;
        entry.last_timestamp = time;
        entry.last_source_unit_state = source;
        entry.last_cast_id = cast_track_id;
        entry.last_target_unit_state = target;
        let mut lines: Vec<String> = vec![];
        if entry.stacks == 1 {
            let mut gained_line = format!(
                "{},{},{},{},{},{},",
                time, "EFFECT_CHANGED", "GAINED", "1", cast_track_id, MOULDERING_TAINT_ID.to_string()
            );
            let rest = parts[9..].join(",");
            gained_line.push_str(&rest);
            lines.push(gained_line);
        }
        let mut line = format!(
            "{},{},{},{},{},{},",
            time, "EFFECT_CHANGED", "UPDATED", entry.stacks, cast_track_id, MOULDERING_TAINT_ID.to_string()
        );
        let rest = parts[9..].join(",");
        line.push_str(&rest);
        lines.push(line);
        // println!("{}", line);
        return Some(lines);
    } else {
        let mut removed = Vec::new();
        for (id, entry) in custom_log_data.taint_stacks.iter_mut() {
            if parts[19] != "*" {
                let target = parse::unit_state(&parts, 19);
                if target.unit_id == *id {
                    if (time > entry.last_timestamp + *MOULDERING_TAINT_TIME as u64 || (event::parse_event_result(&parts[2]).unwrap() == EventResult::Died || target.health == 0)) && entry.stacks > 0 {
                        entry.last_timestamp = time;
                        entry.stacks = 0;
                        let e = entry.last_source_unit_state;
                        let mut line = format!(
                            "{},{},{},{},{},{},{},{}/{},{}/{},{}/{},{}/{},{}/{},{},{},{},{},",
                            time, "EFFECT_CHANGED", "FADED", "1", entry.last_cast_id, MOULDERING_TAINT_ID.to_string(),
                            e.unit_id, e.health, e.max_health, e.magicka, e.max_magicka, e.stamina, e.max_stamina, e.ultimate, e.max_ultimate, e.werewolf, e.werewolf_max, e.shield, e.map_x, e.map_y, e.heading
                        );
                        let rest = parts[19..].join(",");
                        line.push_str(&rest);
                        // println!("{}", line);
                        removed.push(id.clone());
                        return Some(vec![line]);
                    }
                }
            }
            if time > entry.last_timestamp + 500 + *MOULDERING_TAINT_TIME as u64 && entry.stacks > 0 {
                entry.last_timestamp = time;
                entry.stacks = 0;
                let e = entry.last_source_unit_state;
                let t = entry.last_target_unit_state;
                let line = format!(
                    "{},{},{},{},{},{},{},{}/{},{}/{},{}/{},{}/{},{}/{},{},{},{},{},{},{}/{},{}/{},{}/{},{}/{},{}/{},{},{},{},{}",
                    time, "EFFECT_CHANGED", "FADED", "1", entry.last_cast_id, MOULDERING_TAINT_ID.to_string(),
                    e.unit_id, e.health, e.max_health, e.magicka, e.max_magicka, e.stamina, e.max_stamina, e.ultimate, e.max_ultimate, e.werewolf, e.werewolf_max, e.shield, e.map_x, e.map_y, e.heading,
                    t.unit_id, "0", t.max_health, "0", "0", "0", "0", "0", "0", "0", "0", "0", t.map_x, t.map_y, t.heading
                );
                removed.push(id.clone());
                return Some(vec![line]);
            }
        }
        for k in removed {
            custom_log_data.taint_stacks.remove_entry(&k);
        }
        
    }

    return None
}