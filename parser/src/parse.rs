use std::{collections::HashMap, u32};

use esosim_models::player::{EnchantType, GearPiece, GearSlot, GearEnchant};

use crate::{effect::{self, Ability, Effect}, player::{self, Player, effective_level, match_gear_quality, match_gear_trait}, unit::{self, Unit, UnitType, blank_unit_state}};

pub fn is_true(value: &str) -> bool {
    value == "T"
}

fn parse_u32(s: &str) -> u32 {
    s.parse::<u32>().unwrap_or(0)
}

fn parse_f32(s: &str) -> f32 {
    s.parse::<f32>().unwrap_or(0.0)
}

fn parse_pair(s: &str) -> (u32, u32) {
    let mut split = s.splitn(2, '/');
    let first = split.next().map_or(0, parse_u32);
    let second = split.next().map_or(0, parse_u32);
    (first, second)
}

pub fn unit_state(parts: &[String], start_index: usize) -> unit::UnitState {
    if parts.len() < start_index + 10 {
        eprintln!("Impossible unit state: {parts:?}");
        return blank_unit_state();
    }

    let slice = &parts[start_index..start_index + 10];
    let [
        unit_id_str, health_str, magicka_str, stamina_str, ultimate_str,
        werewolf_str, shield_str, map_x_str, map_y_str, heading_str
    ] = match slice {
        [a, b, c, d, e, f, g, h, i, j] => [a, b, c, d, e, f, g, h, i, j],
        _ => {
            eprintln!("Invalid unit state slice: {parts:?}");
            return blank_unit_state();
        }
    };

    let (health, max_health) = parse_pair(health_str);
    let (magicka, max_magicka) = parse_pair(magicka_str);
    let (stamina, max_stamina) = parse_pair(stamina_str);
    let (ultimate, max_ultimate) = parse_pair(ultimate_str);
    let (werewolf, werewolf_max) = parse_pair(werewolf_str);

    unit::UnitState {
        unit_id: parse_u32(unit_id_str),
        health,
        max_health,
        magicka,
        max_magicka,
        stamina,
        max_stamina,
        ultimate,
        max_ultimate,
        werewolf,
        werewolf_max,
        shield: parse_u32(shield_str),
        map_x: parse_f32(map_x_str),
        map_y: parse_f32(map_y_str),
        heading: parse_f32(heading_str),
    }
}

pub fn unit_state_id_only(parts: &[String], start_index: usize) -> Option<u32> {
    if parts.len() <= start_index {
        eprintln!("Impossible unit state: {parts:?}");
        return None;
    }

    Some(parse_u32(&parts[start_index]))
}


pub fn player(parts: &[String]) -> Player {
    let unit_id: u32 = parts[2].parse().unwrap();
    Player {
        unit_id,
        is_local_player: is_true(&parts[4]),
        player_per_session_id: parts[5].parse().unwrap(),
        class_id: player::match_class(&parts[8]),
        race_id: player::match_race(&parts[9]),
        name: parts[10].trim_matches('"').to_string(),
        display_name: parts[11].trim_matches('"').to_string(),
        character_id: parts[12].parse().unwrap(),
        level: parts[13].parse().unwrap(),
        champion_points: parts[14].parse().unwrap(),
        is_grouped_with_local_player: is_true(&parts[17]),
        unit_state: unit::blank_unit_state(),
        effects: Vec::new(),
        gear: player::empty_loadout(),
        primary_abilities: Vec::new(),
        backup_abilities: Vec::new(),
    }
}

pub fn monster(parts: &[String]) -> Unit {
    let unit_id: u32 = parts[2].parse().unwrap();
    Unit {
        unit_id,
        unit_type: UnitType::Monster,
        monster_id: parts[6].parse().unwrap(),
        is_boss: is_true(&parts[7]),
        name: parts[10].trim_matches('"').to_string(),
        level: parts[13].parse().unwrap(),
        champion_points: parts[14].parse().unwrap(),
        owner_unit_id: parts[15].parse().unwrap(),
        reaction: unit::match_reaction(&parts[16]),
        unit_state: unit::blank_unit_state(),
        effects: Vec::new(),
    }
}

pub fn monster_updated(parts: &[String], unit: Unit) -> Unit {
    Unit {
        unit_id: parts[2].parse().unwrap(),
        unit_type: unit.unit_type,
        monster_id: unit.monster_id,
        is_boss: unit.is_boss,
        name: parts[5].trim_matches('"').to_string(),
        level: parts[8].parse().unwrap(),
        champion_points: parts[9].parse().unwrap(),
        owner_unit_id: parts[10].parse().unwrap(),
        reaction: unit::match_reaction(&parts[11]),
        unit_state: unit.unit_state,
        effects: unit.effects,
    }
}

pub fn object(parts: &[String]) -> Unit {
    let unit_id: u32 = parts[2].parse().unwrap();
    Unit {
        unit_id,
        unit_type: UnitType::Object,
        monster_id: 0,
        is_boss: false,
        name: parts[10].trim_matches('"').to_string(),
        level: parts[13].parse().unwrap(),
        champion_points: parts[14].parse().unwrap(),
        owner_unit_id: parts[15].parse().unwrap(),
        reaction: unit::match_reaction(&parts[16]),
        unit_state: unit::blank_unit_state(),
        effects: Vec::new(),
    }
}

pub fn gear_piece(part: &str) -> Option<(GearPiece, GearSlot)> {
    let split: Vec<&str> = part.split(',').collect();
    if split.len() < 3 {println!("Gear piece malformed: {split:?}"); return None}
    let level = split[9].parse::<u8>().unwrap_or(u8::MAX);
    let slot = player::match_gear_slot(split[0]);
    if slot.is_none() {return None}
    let is_cp = split.get(8).is_some_and(|v| is_true(v));
    let quality = split.get(10).map(|v| player::match_gear_quality(v)).unwrap_or(esosim_data::item_type::ItemQuality::Normal);

    let enchant = if level > 0 {
        let mut et = player::match_enchant_type(split[7]);
        if et.is_none() && matches!(slot.as_ref().unwrap(), GearSlot::Ring1 | GearSlot::Necklace | GearSlot::Ring2) {
            // assume enchant is indeko tri recovery
            et = Some(EnchantType::PrismaticRecovery);
        } else if et.is_none() {
            return None
        }

        Some(GearEnchant {
            glyph: et.unwrap(),
            effective_level: level,
            quality: quality,
        })
    } else {
        None
    };

    Some((GearPiece {
        item_id: split[1].parse().unwrap_or(0),
        gear_trait: match_gear_trait(split[4]),
        quality: match_gear_quality(split[5]),
        set_id: Some(split[6].parse().unwrap_or(0)),
        enchant,
        effective_level: effective_level(level, is_cp),
    }, slot.unwrap()))
}

pub fn ability(parts: &[String]) -> Ability {
    let id: u32 = parts[2].parse().unwrap();
    Ability {
        id,
        name: parts[3].trim_matches('"').into(),
        icon: parts[4]
            .trim_matches('"')
            .split('/')
            .next_back()
            .unwrap()
            .replace(".dds", ".png")
            .into(),
        interruptible: is_true(&parts[5]),
        blockable: is_true(&parts[6]),
        scribing: if parts.len() == 10 {
            Some((7..10).map(|i| parts[i].trim_matches('"').to_owned()).collect())
        } else {
            None
        },
    }
}

pub fn effect(parts: &[String], ability_lookup: &HashMap<u32, Ability>) -> Effect {
    let effect_id: u32 = parts[2].parse().unwrap();
    let ability = ability_lookup
        .get(&effect_id)
        .expect("ABILITY_INFO must precede EFFECT_INFO")
        .clone();
    Effect {
        ability,
        stack_count: 0,
        effect_type: effect::parse_effect_type(&parts[3]),
        status_effect_type: effect::parse_status_effect_type(&parts[4]),
        synergy: if parts.len() > 6 {
            parts[6].parse().ok()
        } else {
            None
        },
    }
}

pub fn handle_line(line: &str) -> Vec<String> {
    let mut result = Vec::with_capacity(17);

    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut start = 0usize;
    let mut bracket_level = 0usize;
    let mut in_quotes = false;

    for i in 0..len {
        match bytes[i] {
            b'"' => {
                let mut backslashes = 0usize;
                let mut j = i;
                while j > 0 && bytes[j - 1] == b'\\' {
                    backslashes += 1;
                    j -= 1;
                }
                if backslashes % 2 == 0 {
                    in_quotes = !in_quotes;
                }
            }
            b'[' if !in_quotes => {
                bracket_level += 1;
            }
            b']' if !in_quotes => {
                bracket_level = bracket_level.saturating_sub(1);
            }
            b',' if bracket_level == 0 && !in_quotes => {
                if start < i {
                    process_segment_bytes(line, start, i, &mut result);
                }
                start = i + 1;
            }
            _ => {}
        }
    }

    if start < len {
        process_segment_bytes(line, start, len, &mut result);
    }

    result
}

fn process_segment_bytes(line: &str, start: usize, end: usize, result: &mut Vec<String>) {
    let seg = &line[start..end];
    let trimmed = seg.trim();

    if trimmed.is_empty() {
        return;
    }

    if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed.len() >= 2 {
        process_array_bytes(&trimmed[1..trimmed.len() - 1], result);
    } else {
        result.push(trimmed.to_string());
    }
}

fn process_array_bytes(s: &str, result: &mut Vec<String>) {
    if s.is_empty() {
        result.push(String::new());
        return;
    }

    if !s.contains('[') {
        result.push(s.to_string());
        return;
    }

    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut start = 0usize;
    let mut bracket_level = 0usize;
    let mut in_quotes = false;
    let mut found_any_segments = false;

    for i in 0..len {
        match bytes[i] {
            b'"' => {
                in_quotes = !in_quotes;
            }
            b'[' if !in_quotes => {
                bracket_level += 1;
            }
            b']' if !in_quotes => {
                bracket_level = bracket_level.saturating_sub(1);
            }
            b',' if bracket_level == 0 && !in_quotes => {
                process_nested_segment_bytes(&s[start..i], result);
                found_any_segments = true;
                start = i + 1;
            }
            _ => {}
        }
    }

    if start < len {
        process_nested_segment_bytes(&s[start..len], result);
        found_any_segments = true;
    }

    if !found_any_segments {
        process_nested_segment_bytes(s, result);
    }
}

fn process_nested_segment_bytes(segment: &str, result: &mut Vec<String>) {
    let trimmed = segment.trim();

    if trimmed.is_empty() {
        result.push(String::new());
        return;
    }

    if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed.len() >= 2 {
        process_array_bytes(&trimmed[1..trimmed.len() - 1], result);
    } else {
        result.push(trimmed.to_string());
    }
}