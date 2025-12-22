use std::collections::HashMap;
use esosim_models::player::GearSlot;
use lazy_static::lazy_static;
use esosim_data::item_type::ItemType;

lazy_static! {
    static ref SETS: HashMap<u16, &'static str> = parse_set_data_into_hashmap(); 
    // Set data from https://github.com/Baertram/LibSets/blob/LibSets-reworked/LibSets/Data/
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
    matches!(id, 501 | 503 | 505 | 519 | 520 | 521 | 575 | 576 | 593 | 594 | 596 | 597 | 625 | 626 | 627 | 654 | 655 | 656 | 657 | 658 | 674 | 675 | 676 | 691 | 692 | 693 | 694 | 760 | 761 | 762 | 811 | 812 | 813 | 845)
}

pub fn get_item_type_name(item_type: &ItemType) -> &'static str {
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
    matches!(slot, GearSlot::MainHand | GearSlot::MainHandBackup | GearSlot::OffHand | GearSlot::OffHandBackup)
}

pub fn is_armour_slot(slot: &GearSlot) -> bool {
    matches!(slot, GearSlot::Chest | GearSlot::Head | GearSlot::Shoulders | GearSlot::Hands | GearSlot::Waist | GearSlot::Legs | GearSlot::Feet)
}

pub fn is_jewellery_slot(slot: &GearSlot) -> bool {
    matches!(slot, GearSlot::Necklace | GearSlot::Ring1 | GearSlot::Ring2)
}