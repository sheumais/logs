use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::player::GearSlot;

lazy_static! {
    static ref SETS: HashMap<u16, &'static str> = parse_set_data_into_hashmap(); 
    // Set data from https://github.com/Baertram/LibSets/blob/LibSets-reworked/LibSets/Data/

    static ref ITEM_TYPES: HashMap<u32, &'static str> = parse_set_data_into_hashmap_item_types();
    // Item type data from https://esoitem.uesp.net/viewMinedItems.php
}

pub fn parse_set_data_into_hashmap() -> HashMap<u16, &'static str> {
    let mut lookup_table = HashMap::new();
    let data = include_str!("set_data.txt");
    for line in data.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 2 {
            if let Ok(key) = parts[0].parse::<u16>() {
                lookup_table.insert(key, parts[1]);
            }
        }
    }

    lookup_table
}

pub fn get_set_name(id: u16) -> Option<&'static str> {
    SETS.get(&id).cloned()
}

pub fn is_mythic_set(id: u16) -> bool {
    match id {
        501 => true,
        503 => true,
        505 => true,
        519 => true,
        520 => true,
        521 => true,
        575 => true,
        576 => true,
        593 => true,
        594 => true,
        596 => true,
        597 => true,
        625 => true,
        626 => true,
        627 => true,
        654 => true,
        655 => true,
        656 => true,
        657 => true,
        658 => true,
        674 => true,
        675 => true,
        676 => true,
        691 => true,
        692 => true,
        693 => true,
        694 => true,
        760 => true,
        761 => true,
        762 => true,        
        _ => false,
    }
}


pub fn parse_set_data_into_hashmap_item_types() -> HashMap<u32, &'static str> {
    let mut item_type_table = HashMap::new();
    let data = include_str!("id_to_weapon.txt");

    for line in data.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() > 1 {
            let item_type = parts[0];
            for &id_str in &parts[1..] {
                if let Ok(id) = id_str.parse::<u32>() {
                    item_type_table.insert(id, item_type);
                }
            }
        }
    }

    item_type_table
}

pub fn get_weapon_type_from_hashmap(id: u32) -> &'static str {
    return ITEM_TYPES.get(&id).cloned().unwrap_or("UNKNOWN");
}

pub fn get_weapon_name(name: &'static str) -> &'static str {
    match name {
        "1H_AXE" => "Axe",
        "1H_DAGGER" => "Dagger",
        "1H_MACE" => "Mace",
        "1H_SWORD" => "Sword",
        "2H_AXE" => "2H Axe",
        "2H_MAUL" => "2H Maul",
        "2H_SWORD" => "2H Sword",
        "FROST" => "Frost Staff",
        "INFERNO" => "Inferno Staff",
        "LIGHTNING" => "Lightning Staff",
        "RESTORATION" => "Restoration Staff",
        "SHIELD" => "Shield",
        "BOW" => "Bow",
        _ => "Unknown",
    }
}

pub fn is_weapon_slot(slot: &GearSlot) -> bool {
    match slot {
        GearSlot::MainHand | GearSlot::MainHandBackup | GearSlot::OffHand | GearSlot::OffHandBackup => true,
        _ => false
    }
}