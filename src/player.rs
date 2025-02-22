use crate::unit::{UnitType, UnitState, Effect};


pub struct Player {
    pub unit_id: i32,
    pub unit_type: UnitType,
    pub is_local_player: bool,
    pub player_per_session_id: i32,
    pub class_id: ClassId,
    pub race_id: RaceId,
    pub name: String,
    pub display_name: String,
    pub character_id: i128,
    pub level: i8,
    pub champion_points: i16,
    pub is_grouped_with_local_player: bool,
    pub unit_state: UnitState,
    pub effects: Vec<i32>,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ClassId {
    Dragonknight,
    Sorcerer,
    Nightblade,
    Templar,
    Warden,
    Necromancer,
    Arcanist,
    None,
}

pub fn match_class(string: &str) -> ClassId {
    match string {
        "1" => ClassId::Dragonknight,
        "2" => ClassId::Sorcerer,
        "3" => ClassId::Nightblade,
        "4" => ClassId::Warden,
        "5" => ClassId::Necromancer,
        "6" => ClassId::Templar,
        "117" => ClassId::Arcanist,
        _ => ClassId::None,
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum RaceId {
    Argonian,
    Breton,
    DarkElf,
    HighElf,
    Imperial,
    Khajiit,
    Nord,
    Orc,
    Redguard,
    WoodElf,
    None,
}

pub fn match_race(string: &str) -> RaceId {
    match string {
        "1" => RaceId::Breton,
        "2" => RaceId::Redguard, // ???
        "3" => RaceId::Orc,
        "4" => RaceId::DarkElf,
        "5" => RaceId::Nord,
        "6" => RaceId::Argonian, // ???
        "7" => RaceId::HighElf,
        "8" => RaceId::WoodElf, // ???
        "9" => RaceId::Khajiit,
        "10" => RaceId::Imperial, // ???
        _ => RaceId::None,
    }
}

pub enum GearSlot {
    Head,
    Shoulders,
    Chest,
    Hands,
    Waist,
    Legs,
    Feet,
    Neck,
    Ring1,
    Ring2,
    MainHand,
    MainHandBackup,
    OffHand,
    OffHandBackup,
    None,
}

pub enum GearQuality {
    Normal,
    Fine,
    Superior,
    Epic,
    Legendary,
    Mythic,
    None,
}

pub enum GearTrait {
    Powered,
    Charged,
    Precise,
    Infused,
    Defending,
    Training,
    Sharpened,
    Decisive,
    Sturdy,
    Impenetrable,
    Reinforced,
    WellFitted,
    Invigorating,
    Divines,
    Nirnhoned,
    Healthy,
    Arcane,
    Robust,
    Ornate,
    Intricate,
    Bloodthirsty,
    Harmony,
    Protective,
    Swift,
    Triune,
    None,
}

pub enum EnchantType {
    // todo
    IncreaseSpellDamage,
    PoisonedWeapon,
    IncreasePhysicalDamage,
    FieryWeapon,
    Beserker,
    None
}

pub struct GearEnchant {
    pub enchant_type: EnchantType,
    pub is_enchant_cp: bool,
    pub enchant_level: i8,
    pub enchant_quality: GearQuality,
}

pub struct GearPiece {
    pub slot: GearSlot,
    pub quality: GearQuality,
    pub trait_id: GearTrait,
    pub item_id: i32,
    pub champion_points: i16,
    pub level: i8,
    pub set_id: i32,
    pub enchant: GearEnchant,
}