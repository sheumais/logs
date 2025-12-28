use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::path::Path;
use std::sync::Arc;
use esosim_data::item_type::{GearSlot, ITEM_TYPES, ItemType};
use parser::effect::{is_zen_dot, MOULDERING_TAINT_ID, MOULDERING_TAINT_TIME, ZEN_DEBUFF_ID};
use parser::event::{self, parse_event_result, EventResult};
use parser::parse::{self, gear_piece, unit_state_id_only};
use parser::subclassing::{Subclass, ability_id_to_subclassing, subclass_to_icon, subclass_to_name};
use parser::unit::UnitState;
use parser::{EffectChangedEventType, EventType, UnitAddedEventType};

pub struct CustomLogData {
    pub zen_stacks: HashMap<u32, ZenDebuffState>,
    pub scribing_abilities: Vec<ScribingAbility>,
    pub scribing_map: HashMap<u32, usize>,
    pub scribing_unit_map: HashMap<(Arc<str>, u32), usize>,
    pub taint_stacks: HashMap<u32, MoulderingTaintState>,
    pub units: HashMap<u32, Arc<str>>,
    pub last_combat_event_timestamp: u64,
    pub subclassing_map: HashMap<String, Option<Vec<Subclass>>>,
    pub known_ids: HashMap<u32, bool>,
}

impl Default for CustomLogData {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomLogData {
    pub fn new() -> Self {
        Self {
            zen_stacks: HashMap::new(),
            scribing_abilities: Vec::new(),
            scribing_map: HashMap::new(),
            scribing_unit_map: HashMap::new(),
            taint_stacks: HashMap::new(),
            units: HashMap::new(),
            last_combat_event_timestamp: 0,
            subclassing_map: HashMap::new(),
            known_ids: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.zen_stacks.clear();
        self.taint_stacks.clear();
        self.subclassing_map.clear();
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
    pub name: Arc<str>,
    pub icon: Arc<str>,
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
                log::warn!("Error reading line: {e}");
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
        writeln!(writer, "{line}")
            .map_err(|e| format!("Failed to write line: {e}"))?;
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
        // let is_resurrect = parts.get(2).map(|s| s.as_str()) == Some("SOUL_GEM_RESURRECTION_ACCEPTED");

        let is_zen_debuff = parts
            .get(5)
            .and_then(|s| s.parse::<u32>().ok())
            == Some(*ZEN_DEBUFF_ID);

        if !is_ability && !is_effect && !is_player && !is_zen_debuff {
            modified_lines.push(line);
        }

        modified_lines.extend(new_lines);
    } else {
        let is_resurrect = parts.get(1).map(|s| s.as_str()) == Some("BEGIN_CAST") && parts[5].parse::<u32>() == Ok(26770);
        if is_resurrect {return modified_lines}
        modified_lines.push(line);
    }

    modified_lines
}

fn check_line_for_edits(parts: &[String], custom_log_data: &mut CustomLogData) -> Option<Vec<String>> {
    let event = parts.get(1).map(|s| EventType::from(s.as_str())).unwrap_or(EventType::Unknown);
    match event {
        EventType::EffectChanged => check_effect_changed(parts, &mut custom_log_data.zen_stacks),
        EventType::AbilityInfo => check_ability_info(parts, custom_log_data),
        EventType::EffectInfo => add_arcanist_beam_effect_information(parts, custom_log_data),
        EventType::PlayerInfo => modify_player_data(parts, custom_log_data),
        EventType::CombatEvent => modify_combat_event(parts, custom_log_data),
        EventType::UnitAdded => handle_unit_added(parts, custom_log_data),
        _ => None,
    }
}

const PRAGMATIC: &u32 = &186369;
const EXHAUSTING: &u32 = &186780;

fn check_effect_changed(parts: &[String], zen_hashmap: &mut HashMap<u32, ZenDebuffState>) -> Option<Vec<String>> {
    if parts.len() < 17 {
        return None;
    }
    match &parts[5] {
        id if *id == PRAGMATIC.to_string() => add_arcanist_beam_cast(parts),
        id if *id == EXHAUSTING.to_string() => add_arcanist_beam_cast(parts),
        id if *id == ZEN_DEBUFF_ID.to_string() => add_zen_stacks(parts, zen_hashmap),
        id if is_zen_dot(id.parse::<u32>().unwrap_or(0)) => add_zen_stacks(parts, zen_hashmap),
        _ => None,
    }
}

const MAX_ZEN_STACKS: u8 = 5;

fn add_zen_stacks(parts: &[String], zen_status: &mut HashMap<u32, ZenDebuffState>) -> Option<Vec<String>> {
    let is_zen_debuff = parts[5] == ZEN_DEBUFF_ID.to_string();
    let source_unit_state = unit_state_id_only(parts, 6);
    let source_unit_id = source_unit_state?;
    let target_unit_id = if parts[16] == "*" {
        source_unit_id
    } else {
        let t = unit_state_id_only(parts, 16);
        t?;
        t.unwrap()
    };

    let entry = zen_status.entry(target_unit_id).or_insert_with(|| ZenDebuffState {
        active: false,
        source_id: source_unit_id,
        contributing_ability_ids: Vec::new(),
    });

    if !(is_zen_debuff || source_unit_id == entry.source_id || source_unit_id == target_unit_id) {
        return None
    }

    let event_type = parts.get(2).map(|s| EffectChangedEventType::from(s.as_str())).unwrap_or(EffectChangedEventType::Unknown);
    let ability_id = parts[5].parse::<u32>().unwrap_or(0);

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
            parts[0], parts[1], parts[2], stacks, parts[4], ZEN_DEBUFF_ID
        );
        let rest = parts[6..].join(",");
        line.push_str(&rest);
        return Some(vec![line]);
    } else if source_unit_id == entry.source_id || source_unit_id == target_unit_id {
        match event_type {
            EffectChangedEventType::Gained => {
                if !entry.contributing_ability_ids.contains(&ability_id) {
                    entry.contributing_ability_ids.push(ability_id);
                    if entry.active {
                        let stacks = entry.contributing_ability_ids.len().min(MAX_ZEN_STACKS as usize);
                        let mut line = format!(
                            "{},{},{},{},{},{},",
                            parts[0], parts[1], "UPDATED", stacks, parts[4], ZEN_DEBUFF_ID
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
                            parts[0], parts[1], "UPDATED", stacks, parts[4], ZEN_DEBUFF_ID
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
    None
}

fn check_ability_info(parts: &[String], custom_log_data: &mut CustomLogData) -> Option<Vec<String>> {
    let ability = parse::ability(parts);
    if let Some(ref scribing) = ability.scribing {
        if let Some((existing_index, existing_ability)) = custom_log_data.scribing_abilities
            .iter()
            .enumerate()
            .find(|(_, a)| a.name == ability.name && a.scribing.as_ref() == Some(scribing))
        {
            custom_log_data.scribing_map.insert(ability.id, existing_index);
            let focus_script = &existing_ability.scribing.as_ref().unwrap()[0];
            let signature_script = &existing_ability.scribing.as_ref().unwrap()[1];
            let affix_script = &existing_ability.scribing.as_ref().unwrap()[2];
            let new_name = format!("{} ({} / {})", &existing_ability.name, signature_script, affix_script);
            return Some(vec![
                format!("{},{},{},\"{}\",\"{}\",{},{},\"{}\",\"{}\",\"{}\"",
                    parts[0], "ABILITY_INFO", existing_ability.id, new_name, existing_ability.icon, "F", "T", focus_script, signature_script, affix_script),
                format!("{},{},{},\"{}\",\"{}\",{},{},\"{}\",\"{}\",\"{}\"",
                    parts[0], "ABILITY_INFO", ability.id, ability.name, existing_ability.icon, "F", "T", focus_script, signature_script, affix_script)
            ]);
        }
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
        Some(vec![
            format!("{},{},{},\"{}\",\"{}\",{},{},\"{}\",\"{}\",\"{}\"",
                parts[0], "ABILITY_INFO", scribing_ability.id, new_name, scribing_ability.icon, "F", "T", focus_script, signature_script, affix_script),
            format!("{},{},{},\"{}\",\"{}\",{},{},\"{}\",\"{}\",\"{}\"",
                parts[0], "ABILITY_INFO", ability_id_clone, ability_name_clone, scribing_ability.icon, "F", "T", focus_script, signature_script, affix_script)
        ])
    } else if (parts[2] == PRAGMATIC.to_string() && !custom_log_data.known_ids.contains_key(PRAGMATIC)) || (parts[2] == EXHAUSTING.to_string() && !custom_log_data.known_ids.contains_key(EXHAUSTING)) {
        add_arcanist_beam_information(parts, custom_log_data)
    } else if parts[2].parse::<u32>().ok() == Some(BLOCKADE_DEFAULT) && !custom_log_data.known_ids.contains_key(&BLOCKADE_DEFAULT) {
        add_blockade_versions(parts, custom_log_data)
    } else {
        custom_log_data.known_ids.insert(ability.id, true);
        return None
    }
}

fn add_arcanist_beam_information(parts: &[String], custom_log_data: &mut CustomLogData) -> Option<Vec<String>> {
    let mut lines = Vec::new();
    if parts[2] == PRAGMATIC.to_string() && !custom_log_data.known_ids.contains_key(PRAGMATIC) {
        lines.push(format!("{},{},{},{},{},{},{}", parts[0], parts[1], PRAGMATIC, parts[3], "\"/esoui/art/icons/ability_arcanist_002_b.dds\"", "F", "T"));
        return Some(lines);
    } else if parts[2] == EXHAUSTING.to_string() && !custom_log_data.known_ids.contains_key(EXHAUSTING) {
        lines.push(format!("{},{},{},{},{},{},{}", parts[0], parts[1], EXHAUSTING, parts[3], "\"/esoui/art/icons/ability_arcanist_002_a.dds\"", "F", "T"));
        return Some(lines);
    }
    None
}

fn add_arcanist_beam_effect_information(parts: &[String], custom_log_data: &mut CustomLogData) -> Option<Vec<String>> {
    let mut lines = Vec::new();
    if parts[2] == PRAGMATIC.to_string() && !custom_log_data.known_ids.contains_key(PRAGMATIC) {
        lines.push(format!("{},{},{},{},{},{}", parts[0], "EFFECT_INFO", PRAGMATIC, "BUFF", "NONE", "NEVER"));
        custom_log_data.known_ids.insert(*PRAGMATIC, true);
        return Some(lines);
    } else if parts[2] == EXHAUSTING.to_string() && !custom_log_data.known_ids.contains_key(EXHAUSTING) {
        lines.push(format!("{},{},{},{},{},{}", parts[0], "EFFECT_INFO", EXHAUSTING, "BUFF", "NONE", "NEVER"));
        custom_log_data.known_ids.insert(*EXHAUSTING, true);
        return Some(lines);
    }
    None
}

const BLOCKADE_FIRE: u32 = 39012;
const BLOCKADE_STORMS: u32 = 39018;
const BLOCKADE_FROST: u32 = 39028;
const BLOCKADE_DEFAULT: u32 = 39011;

fn add_blockade_versions(parts: &[String], custom_log_data: &mut CustomLogData) -> Option<Vec<String>> {
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
    custom_log_data.known_ids.insert(BLOCKADE_FIRE, true);
    custom_log_data.known_ids.insert(BLOCKADE_STORMS, true);
    custom_log_data.known_ids.insert(BLOCKADE_FROST, true);
    custom_log_data.known_ids.insert(BLOCKADE_DEFAULT, true);
    Some(lines)
}

fn modify_player_data(parts: &[String], custom_log_data: &mut CustomLogData) -> Option<Vec<String>> {
    
    // log::trace!("Modifying player data: {:?}", parts);

    if parts.len() < 7 { // this should never occur
        return None;
    }

    let mut primary_ability_id_list: Vec<u32> = parts[parts.len() - 2].split(',').map(|x| x.parse::<u32>().unwrap_or_default()).collect();
    let mut backup_ability_id_list: Vec<u32> = parts[parts.len() - 1].split(',').map(|x| x.parse::<u32>().unwrap_or_default()).collect();

    let mut frontbar_type = ItemType::Unknown;
    let mut backbar_type = ItemType::Unknown;
    let gear_parts: Vec<&str> = parts[5..parts.len()-2].iter().map(|s| s.as_str()).collect();

    let mut processed_gear: Vec<String> = Vec::new();
    let mut cryptcanon = false;
    for i in gear_parts {
        let gear = gear_piece(i);
        if let Some((gear_piece, slot)) = gear {
            if gear_piece.item_id == 194509 {cryptcanon = true}
            // let is_mythic = is_mythic_set(gear_piece_obj.set_id);
            let gear_str = i.to_string();
            // if is_mythic { // save for the rainy day where esologs adds functionality for mythic items.. https://discord.com/channels/503331371159257089/714906580646232135/878731437807902760
            //     if let Some(pos) = gear_str.find("LEGENDARY") {
            //         gear_str.replace_range(pos..pos + "LEGENDARY".len(), "MYTHIC_OVERRIDE");
            //     }
            // }
            processed_gear.push(format!("[{gear_str}]"));
            let item_slot = slot;
            let item_type = ITEM_TYPES.get(&gear_piece.item_id);
            if let Some(item) = item_type {
                if item_slot == GearSlot::MainHand {
                    frontbar_type = *item;
                } else if item_slot == GearSlot::MainHandBackup {
                    backbar_type = *item;
                }
            }
        }
    }

    if cryptcanon {
        if primary_ability_id_list.contains(&195031) || backup_ability_id_list.contains(&195031) {cryptcanon = false} else if primary_ability_id_list.len() == 6 && backup_ability_id_list.len() == 6 {
            if let Some(last) = primary_ability_id_list.last_mut() {
                *last = 195031;
            }
            if let Some(last) = backup_ability_id_list.last_mut() {
                *last = 195031;
            }
        }
    }

    let player_id = parts[2].parse::<u32>().unwrap();
    let player_name = custom_log_data.units.get(&player_id).cloned().unwrap_or_else(|| player_id.to_string().into());

    for id in &mut primary_ability_id_list {
        if matches!(*id, BLOCKADE_DEFAULT | BLOCKADE_FIRE | BLOCKADE_FROST | BLOCKADE_STORMS) {
            *id = match frontbar_type {
                ItemType::FrostStaff => BLOCKADE_FROST,
                ItemType::FireStaff => BLOCKADE_FIRE,
                ItemType::LightningStaff => BLOCKADE_STORMS,
                _ => BLOCKADE_DEFAULT,
            };
        } else if let Some(index) = custom_log_data.scribing_unit_map.get(&(player_name.clone(), *id)) {
            // log::trace!("Setting {} to scribing {:?} for {}", player_name, custom_log_data.scribing_abilities[*index].scribing, id);
            *id = BEGIN_SCRIBING_ABILITIES + *index as u32;
        } else if custom_log_data.scribing_map.contains_key(id) {
            if let Some(index) = custom_log_data.scribing_map.get(id) {
                custom_log_data.scribing_unit_map.insert((player_name.clone(), *id), *index);
                *id = BEGIN_SCRIBING_ABILITIES + *index as u32;
            }
        }
    }

    for id in &mut backup_ability_id_list {
        if matches!(*id, BLOCKADE_DEFAULT | BLOCKADE_FIRE | BLOCKADE_FROST | BLOCKADE_STORMS) {
            *id = match backbar_type {
                ItemType::FrostStaff => BLOCKADE_FROST,
                ItemType::FireStaff => BLOCKADE_FIRE,
                ItemType::LightningStaff => BLOCKADE_STORMS,
                _ => BLOCKADE_DEFAULT,
            };
        } else if let Some(index) = custom_log_data.scribing_unit_map.get(&(player_name.clone(), *id)) {
            // log::trace!("Setting {} to scribing {:?} for {}", player_name, custom_log_data.scribing_abilities[*index].scribing, id);
            *id = BEGIN_SCRIBING_ABILITIES + *index as u32;
        } else if custom_log_data.scribing_map.contains_key(id) {
            if let Some(index) = custom_log_data.scribing_map.get(id) {
                custom_log_data.scribing_unit_map.insert((player_name.clone(), *id), *index);
                *id = BEGIN_SCRIBING_ABILITIES + *index as u32;
            }
        }
    }

    let mut result = Vec::new();

    let mut long_term_buffs: Vec<u32> = parts[3].split(',').map(|x| x.parse::<u32>().unwrap_or_default()).collect();
    let mut long_term_buff_stacks: Vec<u8> = parts[4].split(',').map(|x| x.parse::<u8>().unwrap_or_default()).collect();
    let mut subclasses_to_append = Vec::new();
    for ability_id in long_term_buffs.iter().chain(primary_ability_id_list.iter()).chain(backup_ability_id_list.iter()) {
        if let Some(subclass) = ability_id_to_subclassing(*ability_id) {
            if !custom_log_data.known_ids.contains_key(&(subclass as u32)) {
                let subclass_definition = format!(
                    "{},ABILITY_INFO,{},\"Subclass: {}\",\"{}\",F,T",
                    parts[0],
                    subclass as u32,
                    subclass_to_name(subclass),
                    subclass_to_icon(subclass)
                );
                let subclass_effect_info = format!(
                    "{},EFFECT_INFO,{},BUFF,NONE,DEFAULT",
                    parts[0],
                    subclass as u32
                );
                result.push(subclass_definition);
                result.push(subclass_effect_info);
                custom_log_data.known_ids.insert(subclass as u32, true);
            }
            subclasses_to_append.push(subclass);
        }
    }
    subclasses_to_append.sort_unstable();
    subclasses_to_append.dedup();
    for subclass in &subclasses_to_append {
        long_term_buffs.push(subclass.clone() as u32);
        long_term_buff_stacks.push(1);
    }
    custom_log_data.subclassing_map.insert(player_name.to_string(), Some(subclasses_to_append));
    
    let mut new_parts: Vec<String> = vec![
        format!("{}", parts[0]),
        format!("{}", parts[1]),
        format!("{}", parts[2]),
        format!("[{}]", long_term_buffs.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")),
        format!("[{}]", long_term_buff_stacks.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")),
    ];
    new_parts.push(format!("[{}]", processed_gear.join(",")));
    new_parts.push(format!("[{}]", primary_ability_id_list.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")));
    new_parts.push(format!("[{}]", backup_ability_id_list.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")));

    if cryptcanon && !custom_log_data.known_ids.contains_key(&195031) {
        result.push(format!("{},ABILITY_INFO,195031,\"Crypt Transfer\",\"/esoui/art/icons/u38_ability_armor_ultimatetransfer.dds\",F,T", parts[0]));
        custom_log_data.known_ids.insert(195031, true);
    }
    result.push(new_parts.join(","));
    Some(result)
}

fn modify_combat_event(parts: &[String], custom_log_data: &mut CustomLogData) -> Option<Vec<String>> {
    let ability_id = match parts[8].parse::<u32>() {
        Ok(id) => id,
        Err(_) => return None,
    };
    let time = match parts[0].parse::<u64>() {
        Ok(t) => t,
        Err(_) => return None,
    };

    let event_type = parse_event_result(&parts[2]);

    if event_type == Some(EventResult::SoulGemResurrectionAccepted) {
        let mut lines = Vec::new();
        let mut line = format!("{},BEGIN_CAST,0,F,0,26770,", parts[0]);
        line.push_str(&parts[9..].join(","));
        lines.push(line);
        lines.push(format!("{},END_CAST,COMPLETED,0,26770", parts[0]));
        return Some(lines);
    }

    let mo_taint_time = *MOULDERING_TAINT_TIME as u64;

    match ability_id {
        id if id == *MOULDERING_TAINT_ID => {
            let source = parse::unit_state(parts, 9);
            let target = parse::unit_state(parts, 19);
            let cast_track_id = parts[7].parse::<u32>().unwrap();

            let entry = custom_log_data.taint_stacks.entry(target.unit_id)
                .or_insert_with(|| MoulderingTaintState {
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
                    time, "EFFECT_CHANGED", "GAINED", "1", cast_track_id, MOULDERING_TAINT_ID
                );
                gained_line.push_str(&parts[9..].join(","));
                lines.push(gained_line);
            }

            let mut line = format!(
                "{},{},{},{},{},{},",
                time, "EFFECT_CHANGED", "UPDATED", entry.stacks, cast_track_id, MOULDERING_TAINT_ID
            );
            line.push_str(&parts[9..].join(","));
            lines.push(line);
            return Some(lines);
        },
        _ => {
            let mut removed = Vec::new();
            if custom_log_data.taint_stacks.is_empty() {
                return None;
            }

            for (id, entry) in custom_log_data.taint_stacks.iter_mut() {
                if parts[19] != "*" && parse::unit_state_id_only(parts, 19) == Some(*id) {
                    let target = parse::unit_state(parts, 19);
                    if (time > entry.last_timestamp + mo_taint_time || (event::parse_event_result(&parts[2]).unwrap() == EventResult::Died || target.health == 0)) && entry.stacks > 0 {
                        entry.last_timestamp = time;
                        entry.stacks = 0;
                        let e = entry.last_source_unit_state;
                        let mut line = format!(
                            "{},{},{},{},{},{},{},{}/{},{}/{},{}/{},{}/{},{}/{},{},{},{},{},",
                            time, "EFFECT_CHANGED", "FADED", "1", entry.last_cast_id, MOULDERING_TAINT_ID,
                            e.unit_id, e.health, e.max_health, e.magicka, e.max_magicka, e.stamina, e.max_stamina, e.ultimate, e.max_ultimate, e.werewolf, e.werewolf_max, e.shield, e.map_x, e.map_y, e.heading
                        );
                        let rest = parts[19..].join(",");
                        line.push_str(&rest);
                        removed.push(*id);
                        return Some(vec![line]);
                    }
                }
                if time > entry.last_timestamp + mo_taint_time && entry.stacks > 0 {
                    entry.last_timestamp = time;
                    entry.stacks = 0;
                    let e = entry.last_source_unit_state;
                    let t = entry.last_target_unit_state;
                    let line = format!(
                        "{},{},{},{},{},{},{},{}/{},{}/{},{}/{},{}/{},{}/{},{},{},{},{},{},{}/{},{}/{},{}/{},{}/{},{}/{},{},{},{},{}",
                        time, "EFFECT_CHANGED", "FADED", "1", entry.last_cast_id, MOULDERING_TAINT_ID,
                        e.unit_id, e.health, e.max_health, e.magicka, e.max_magicka, e.stamina, e.max_stamina, e.ultimate, e.max_ultimate, e.werewolf, e.werewolf_max, e.shield, e.map_x, e.map_y, e.heading,
                        t.unit_id, "0", t.max_health, "0", "0", "0", "0", "0", "0", "0", "0", "0", t.map_x, t.map_y, t.heading
                    );
                    removed.push(*id);
                    return Some(vec![line]);
                }
            }
            for k in removed {
                custom_log_data.taint_stacks.remove_entry(&k);
            }
        }
    }

    None
}

fn handle_unit_added(parts: &[String], custom_log_data: &mut CustomLogData) -> Option<Vec<String>> {
    let event = UnitAddedEventType::from(parts.get(3).unwrap().as_str());
        match event {
            UnitAddedEventType::Player => {
                let player = parse::player(parts);
                let name: Arc<str> = player.name.into();
                if name == "Offline".into() || name.len() < 3 {return None;}
                custom_log_data.units.insert(player.unit_id, name.clone());
                custom_log_data.units.insert(player.player_per_session_id, name);
            },
            UnitAddedEventType::Monster => {
                // let monster = parse::monster(parts);
                // custom_log_data.units.insert(monster.unit_id, monster.name.clone());
                // custom_log_data.units.insert(monster.unit_id, monster.name);
            },
            UnitAddedEventType::Object | UnitAddedEventType::SiegeWeapon => {
                // let object = parse::object(parts);
            },
            UnitAddedEventType::Unknown => {
                log::error!("Unknown unit added unit type");
            },
        }

    None
}