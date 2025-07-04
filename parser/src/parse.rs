use std::collections::HashMap;

use crate::{effect::{self, Ability, Effect}, player::{self, GearPiece, Player}, unit::{self, Unit, UnitType}};

pub fn is_true(value: &str) -> bool {
    value == "T"
}

pub fn unit_state(parts: &[&str], start_index: usize) -> unit::UnitState {
    let parse_value = |s: &str| s.parse::<u32>().unwrap_or(0);
    let parse_pair = |s: &str| {
        let mut split = s.split('/');
        (
            split.next().map_or(0, parse_value),
            split.next().map_or(0, parse_value),
        )
    };

    unit::UnitState {
        unit_id: parts[start_index].parse::<u32>().unwrap_or(0),
        health: parse_pair(parts[start_index + 1]).0,
        max_health: parse_pair(parts[start_index + 1]).1,
        magicka: parse_pair(parts[start_index + 2]).0,
        max_magicka: parse_pair(parts[start_index + 2]).1,
        stamina: parse_pair(parts[start_index + 3]).0,
        max_stamina: parse_pair(parts[start_index + 3]).1,
        ultimate: parse_pair(parts[start_index + 4]).0,
        max_ultimate: parse_pair(parts[start_index + 4]).1,
        werewolf: parse_pair(parts[start_index + 5]).0,
        werewolf_max: parse_pair(parts[start_index + 5]).1,
        shield: parts[start_index + 6].parse::<u32>().unwrap_or(0),
        map_x: parts[start_index + 7].parse::<f32>().unwrap_or(0.0),
        map_y: parts[start_index + 8].parse::<f32>().unwrap_or(0.0),
        heading: parts[start_index + 9].parse::<f32>().unwrap_or(0.0),
    }
}

pub fn player(parts: &[&str]) -> Player {
    let unit_id: u32 = parts[2].parse().unwrap();
    Player {
        unit_id,
        is_local_player: is_true(parts[4]),
        player_per_session_id: parts[5].parse().unwrap(),
        class_id: player::match_class(parts[8]),
        race_id: player::match_race(parts[9]),
        name: parts[10].to_string(),
        display_name: parts[11].trim_matches('"').to_string(),
        character_id: parts[12].parse().unwrap(),
        level: parts[13].parse().unwrap(),
        champion_points: parts[14].parse().unwrap(),
        is_grouped_with_local_player: is_true(parts[17]),
        unit_state: unit::blank_unit_state(),
        effects: Vec::new(),
        gear: player::empty_loadout(),
        primary_abilities: Vec::new(),
        backup_abilities: Vec::new(),
    }
}

pub fn monster(parts: &[&str]) -> Unit {
    let unit_id: u32 = parts[2].parse().unwrap();
    Unit {
        unit_id,
        unit_type: UnitType::Monster,
        monster_id: parts[6].parse().unwrap(),
        is_boss: is_true(parts[7]),
        name: parts[10].trim_matches('"').to_string(),
        level: parts[13].parse().unwrap(),
        champion_points: parts[14].parse().unwrap(),
        owner_unit_id: parts[15].parse().unwrap(),
        reaction: unit::match_reaction(parts[16]),
        unit_state: unit::blank_unit_state(),
        effects: Vec::new(),
    }
}

pub fn monster_updated(parts: &[&str], unit: Unit) -> Unit {
    Unit {
        unit_id: parts[2].parse().unwrap(),
        unit_type: unit.unit_type,
        monster_id: unit.monster_id,
        is_boss: unit.is_boss,
        name: parts[5].trim_matches('"').to_string(),
        level: parts[8].parse().unwrap(),
        champion_points: parts[9].parse().unwrap(),
        owner_unit_id: parts[10].parse().unwrap(),
        reaction: unit::match_reaction(parts[11]),
        unit_state: unit.unit_state,
        effects: unit.effects,
    }
}

pub fn object(parts: &[&str]) -> Unit {
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
        reaction: unit::match_reaction(parts[16]),
        unit_state: unit::blank_unit_state(),
        effects: Vec::new(),
    }
}

pub fn gear_piece(part: &str) -> GearPiece {
    let split: Vec<&str> = part.split(',').collect();
    if player::match_gear_slot(split[0]) == player::GearSlot::None {
        println!("Unknown gear slot: {part}");
    }
    if player::match_gear_trait(split[4]) == player::GearTrait::None && split[4] != "NONE" {
        println!("Unknown gear trait: {part}");
    }
    if player::match_gear_quality(split[5]) == player::GearQuality::None {
        println!("Unknown gear quality: {part}");
    }
    if player::match_enchant_type(split[7]) == player::EnchantType::None {
        println!("Unknown enchant type: {part}");
    }

    GearPiece {
        slot: player::match_gear_slot(split[0]),
        item_id: split[1].parse().unwrap(),
        is_cp: is_true(split[2]),
        level: split[3].parse().unwrap(),
        gear_trait: player::match_gear_trait(split[4]),
        quality: player::match_gear_quality(split[5]),
        set_id: split[6].parse().unwrap(),
        enchant: player::GearEnchant {
            enchant_type: player::match_enchant_type(split[7]),
            is_cp: is_true(split[8]),
            enchant_level: split[9].parse().unwrap(),
            enchant_quality: player::match_gear_quality(split[10]),
        },
    }
}

pub fn ability(parts: &[&str]) -> Ability {
    let id: u32 = parts[2].parse().unwrap();
    Ability {
        id,
        name: parts[3].trim_matches('"').to_string(),
        icon: parts[4]
            .trim_matches('"')
            .split('/')
            .last()
            .unwrap()
            .replace(".dds", ".png"),
        interruptible: is_true(parts[5]),
        blockable: is_true(parts[6]),
        scribing: if parts.len() == 10 {
            Some((7..10).map(|i| parts[i].trim_matches('"').to_owned()).collect())
        } else {
            None
        },
    }
}

pub fn effect(parts: &[&str], ability_lookup: &HashMap<u32, Ability>) -> Effect {
    let effect_id: u32 = parts[2].parse().unwrap();
    let ability = ability_lookup
        .get(&effect_id)
        .expect("ABILITY_INFO must precede EFFECT_INFO")
        .clone();
    Effect {
        ability,
        stack_count: 0,
        effect_type: effect::parse_effect_type(parts[3]),
        status_effect_type: effect::parse_status_effect_type(parts[4]),
        synergy: if parts.len() > 6 {
            parts[6].parse().ok()
        } else {
            None
        },
    }
}