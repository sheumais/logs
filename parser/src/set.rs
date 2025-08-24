use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::player::GearSlot;

lazy_static! {
    static ref SETS: HashMap<u16, &'static str> = parse_set_data_into_hashmap(); 
    // Set data from https://github.com/Baertram/LibSets/blob/LibSets-reworked/LibSets/Data/

    static ref ITEM_TYPES: HashMap<u32, &'static str> = parse_item_types_into_hashmap();
    // Item type from game using https://github.com/sheumais/ItemTypeDataExtractTool
}

pub fn parse_set_data_into_hashmap() -> HashMap<u16, &'static str> {
    let mut lookup_table = HashMap::new();
    let data = include_str!("set_data.csv");
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
        811 => true,
        812 => true,
        813 => true,
        _ => false,
    }
}


pub fn parse_item_types_into_hashmap() -> HashMap<u32, &'static str> {
    let mut item_type_table = HashMap::new();
    let data = include_str!("item_data.csv");

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

#[derive(PartialEq, Debug)]
pub enum ItemType {
    Axe,
    Dagger,
    Mace,
    Sword,
    TwoHandedAxe,
    TwoHandedMace,
    TwoHandedSword,
    FrostStaff,
    FireStaff,
    LightningStaff,
    HealingStaff,
    Shield,
    Bow,
    Light,
    Medium,
    Heavy,
    Mara,
    Unknown,
}

pub fn get_item_type_from_hashmap(id: u32) -> ItemType {
    match ITEM_TYPES.get(&id).map(|s| *s) {
        Some("AXE") => ItemType::Axe,
        Some("DAGGER") => ItemType::Dagger,
        Some("MACE") => ItemType::Mace,
        Some("SWORD") => ItemType::Sword,
        Some("TWO_HANDED_AXE") => ItemType::TwoHandedAxe,
        Some("TWO_HANDED_MACE") => ItemType::TwoHandedMace,
        Some("TWO_HANDED_SWORD") => ItemType::TwoHandedSword,
        Some("FROST_STAFF") => ItemType::FrostStaff,
        Some("FIRE_STAFF") => ItemType::FireStaff,
        Some("LIGHTNING_STAFF") => ItemType::LightningStaff,
        Some("HEALING_STAFF") => ItemType::HealingStaff,
        Some("SHIELD") => ItemType::Shield,
        Some("BOW") => ItemType::Bow,
        Some("LIGHT") => ItemType::Light,
        Some("MEDIUM") => ItemType::Medium,
        Some("HEAVY") => ItemType::Heavy,
        Some("MARA") => ItemType::Mara,
        _ => ItemType::Unknown,
    }
}

pub fn get_item_type_name(item_type: ItemType) -> &'static str {
    match item_type {
        ItemType::Axe => "Axe",
        ItemType::Dagger => "Dagger",
        ItemType::Mace => "Mace",
        ItemType::Sword => "Sword",
        ItemType::TwoHandedAxe => "2H Axe",
        ItemType::TwoHandedMace => "2H Maul",
        ItemType::TwoHandedSword => "2H Sword",
        ItemType::FrostStaff => "Ice Staff",
        ItemType::FireStaff => "Inferno Staff",
        ItemType::LightningStaff => "Lightning Staff",
        ItemType::HealingStaff => "Restoration Staff",
        ItemType::Shield => "Shield",
        ItemType::Bow => "Bow",
        ItemType::Light => "Light",
        ItemType::Medium => "Medium",
        ItemType::Heavy => "Heavy",
        ItemType::Mara => "Ring of Mara",
        ItemType::Unknown => "Unknown",
    }
}

pub fn is_weapon_slot(slot: &GearSlot) -> bool {
    match slot {
        GearSlot::MainHand | GearSlot::MainHandBackup | GearSlot::OffHand | GearSlot::OffHandBackup => true,
        _ => false
    }
}

pub fn is_armour_slot(slot: &GearSlot) -> bool {
    match slot {
        GearSlot::Chest | GearSlot::Head | GearSlot::Shoulders | GearSlot::Hands | GearSlot::Waist | GearSlot::Legs | GearSlot::Feet => true,
        _ => false
    }
}

pub fn is_jewellery_slot(slot: &GearSlot) -> bool {
    match slot {
        GearSlot::Necklace | GearSlot::Ring1 | GearSlot::Ring2 => true,
        _ => false
    }
}