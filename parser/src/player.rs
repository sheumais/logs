use std::fmt;

use crate::{effect::Ability, unit::UnitState};

#[derive(Debug, PartialEq, Clone)]
pub struct Player {
    pub unit_id: u32,
    pub is_local_player: bool,
    pub player_per_session_id: u32,
    pub class_id: Class,
    pub race_id: Race,
    pub name: String,
    pub display_name: String,
    pub character_id: u64,
    pub level: u8,
    pub champion_points: u16,
    pub is_grouped_with_local_player: bool,
    pub unit_state: UnitState,
    pub effects: Vec<u32>,
    pub gear: Loadout,
    pub primary_abilities: Vec<Ability>,
    pub backup_abilities: Vec<Ability>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Loadout {
    pub head: GearPiece,
    pub shoulders: GearPiece,
    pub chest: GearPiece,
    pub hands: GearPiece,
    pub waist: GearPiece,
    pub legs: GearPiece,
    pub feet: GearPiece,
    pub necklace: GearPiece,
    pub ring1: GearPiece,
    pub ring2: GearPiece,
    pub main_hand: GearPiece,
    pub main_hand_backup: GearPiece,
    pub poison: GearPiece,
    pub off_hand: GearPiece,
    pub off_hand_backup: GearPiece,
    pub backup_poison: GearPiece,
}

impl Player {
    pub fn insert_gear_piece(&mut self, gear_piece: GearPiece) {
        let slot = gear_piece.slot.clone();
        self.gear.insert(slot, gear_piece);
    }
}

impl Loadout {
    pub fn insert(&mut self, slot: GearSlot, gear_piece: GearPiece) {
        match slot {
            GearSlot::Head => self.head = gear_piece,
            GearSlot::Shoulders => self.shoulders = gear_piece,
            GearSlot::Chest => self.chest = gear_piece,
            GearSlot::Hands => self.hands = gear_piece,
            GearSlot::Waist => self.waist = gear_piece,
            GearSlot::Legs => self.legs = gear_piece,
            GearSlot::Feet => self.feet = gear_piece,
            GearSlot::Necklace => self.necklace = gear_piece,
            GearSlot::Ring1 => self.ring1 = gear_piece,
            GearSlot::Ring2 => self.ring2 = gear_piece,
            GearSlot::MainHand => self.main_hand = gear_piece,
            GearSlot::MainHandBackup => self.main_hand_backup = gear_piece,
            GearSlot::OffHand => self.off_hand = gear_piece,
            GearSlot::OffHandBackup => self.off_hand_backup = gear_piece,
            GearSlot::Poison => self.poison = gear_piece,
            GearSlot::BackupPoison => self.backup_poison = gear_piece,
            _ => {}
        }
    }
}

pub fn empty_gear_piece() -> GearPiece {
    GearPiece {
        slot: GearSlot::None,
        item_id: 0,
        is_cp: false,
        level: 0,
        gear_trait: GearTrait::None,
        quality: GearQuality::None,
        set_id: 0,
        enchant: None,
    }
}

pub fn empty_loadout() -> Loadout {
    Loadout {
        head: empty_gear_piece(),
        shoulders: empty_gear_piece(),
        chest: empty_gear_piece(),
        hands: empty_gear_piece(),
        waist: empty_gear_piece(),
        legs: empty_gear_piece(),
        feet: empty_gear_piece(),
        necklace: empty_gear_piece(),
        ring1: empty_gear_piece(),
        ring2: empty_gear_piece(),
        main_hand: empty_gear_piece(),
        main_hand_backup: empty_gear_piece(),
        poison: empty_gear_piece(),
        off_hand: empty_gear_piece(),
        off_hand_backup: empty_gear_piece(),
        backup_poison: empty_gear_piece(),
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Class {
    Dragonknight,
    Sorcerer,
    Nightblade,
    Templar,
    Warden,
    Necromancer,
    Arcanist,
    None,
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Class::None => "0",
            Class::Dragonknight => "1",
            Class::Sorcerer => "2",
            Class::Nightblade => "3",
            Class::Warden => "4",
            Class::Necromancer => "5",
            Class::Templar => "6",
            Class::Arcanist => "117",
        };
        write!(f, "{s}")
    }
}

pub fn match_class(string: &str) -> Class {
    match string {
        "1" => Class::Dragonknight,
        "2" => Class::Sorcerer,
        "3" => Class::Nightblade,
        "4" => Class::Warden,
        "5" => Class::Necromancer,
        "6" => Class::Templar,
        "117" => Class::Arcanist,
        _ => Class::None,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Race {
    None,
    Breton,
    Redguard,
    Orc,
    DarkElf,
    Nord,
    Argonian,
    HighElf,
    WoodElf,
    Khajiit,
    Imperial,
}

impl fmt::Display for Race {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Race::Breton => "1",
            Race::Redguard => "2",
            Race::Orc => "3",
            Race::DarkElf => "4",
            Race::Nord => "5",
            Race::Argonian => "6",
            Race::HighElf => "7",
            Race::WoodElf => "8",
            Race::Khajiit => "9",
            Race::Imperial => "10",
            Race::None => "0",
        };
        write!(f, "{s}")
    }
}

pub fn match_race(string: &str) -> Race {
    match string {
        "1" => Race::Breton,
        "2" => Race::Redguard,
        "3" => Race::Orc,
        "4" => Race::DarkElf,
        "5" => Race::Nord,
        "6" => Race::Argonian,
        "7" => Race::HighElf,
        "8" => Race::WoodElf,
        "9" => Race::Khajiit,
        "10" => Race::Imperial,
        _ => Race::None,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum GearSlot {
    Head,
    Shoulders,
    Chest,
    Hands,
    Waist,
    Legs,
    Feet,
    Necklace,
    Ring1,
    Ring2,
    MainHand,
    MainHandBackup,
    Poison,
    OffHand,
    OffHandBackup,
    BackupPoison,
    Costume,
    None,
}

pub fn match_gear_slot(string: &str) -> GearSlot {
    match string {
        "HEAD" => GearSlot::Head,
        "SHOULDERS" => GearSlot::Shoulders,
        "CHEST" => GearSlot::Chest,
        "HAND" => GearSlot::Hands,
        "WAIST" => GearSlot::Waist,
        "LEGS" => GearSlot::Legs,
        "FEET" => GearSlot::Feet,
        "NECK" => GearSlot::Necklace,
        "RING1" => GearSlot::Ring1,
        "RING2" => GearSlot::Ring2,
        "MAIN_HAND" => GearSlot::MainHand,
        "OFF_HAND" => GearSlot::OffHand,
        "BACKUP_MAIN" => GearSlot::MainHandBackup,
        "BACKUP_OFF" => GearSlot::OffHandBackup,
        "COSTUME" => GearSlot::Costume,
        "POISON" => GearSlot::Poison,
        "BACKUP_POISON" => GearSlot::BackupPoison,
        _ => GearSlot::None,
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GearQuality {
    Trash,
    Normal,
    Magic,
    Arcane,
    Artifact,
    Legendary,
    Mythic,
    None,
}

pub fn match_gear_quality(string: &str) -> GearQuality {
    match string {
        "TRASH" => GearQuality::Trash,
        "NORMAL" => GearQuality::Normal,
        "MAGIC" => GearQuality::Magic,
        "ARCANE" => GearQuality::Arcane,
        "ARTIFACT" => GearQuality::Artifact,
        "LEGENDARY" => GearQuality::Legendary,
        "MYTHIC" => GearQuality::Mythic,
        _ => GearQuality::None,
    }
}

#[derive(Debug, PartialEq, Clone)]
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
    Bloodthirsty,
    Harmony,
    Protective,
    Swift,
    Triune,
    Intricate,
    None,
}

pub fn match_gear_trait(string: &str) -> GearTrait {
    match string {
        "JEWELRY_BLOODTHIRSTY" => GearTrait::Bloodthirsty,
        "JEWELRY_HARMONY" => GearTrait::Harmony,
        "JEWELRY_PROTECTIVE" => GearTrait::Protective,
        "JEWELRY_SWIFT" => GearTrait::Swift,
        "JEWELRY_TRIUNE" => GearTrait::Triune,
        "JEWELRY_INFUSED" => GearTrait::Infused,
        "JEWELRY_ARCANE" => GearTrait::Arcane,
        "JEWELRY_ROBUST" => GearTrait::Robust,
        "JEWELRY_HEALTHY" => GearTrait::Healthy,

        "ARMOR_STURDY" => GearTrait::Sturdy,
        "ARMOR_IMPENETRABLE" => GearTrait::Impenetrable,
        "ARMOR_REINFORCED" => GearTrait::Reinforced,
        "ARMOR_WELL_FITTED" => GearTrait::WellFitted,
        "ARMOR_DIVINES" => GearTrait::Divines,
        "ARMOR_NIRNHONED" => GearTrait::Nirnhoned,
        "ARMOR_INFUSED" => GearTrait::Infused,
        "ARMOR_TRAINING" => GearTrait::Training,
        "ARMOR_PROSPEROUS" => GearTrait::Invigorating,

        "WEAPON_INFUSED" => GearTrait::Infused,
        "WEAPON_NIRNHONED" => GearTrait::Nirnhoned,
        "WEAPON_CHARGED" => GearTrait::Charged,
        "WEAPON_DECISIVE" => GearTrait::Decisive,
        "WEAPON_DEFENDING" => GearTrait::Defending,
        "WEAPON_POWERED" => GearTrait::Powered,
        "WEAPON_PRECISE" => GearTrait::Precise,
        "WEAPON_SHARPENED" => GearTrait::Sharpened,
        "WEAPON_TRAINING" => GearTrait::Training,

        "ARMOR_INTRICATE" => GearTrait::Intricate,
        "WEAPON_INTRICATE" => GearTrait::Intricate,
        "JEWELRY_INTRICATE" => GearTrait::Intricate,
        _ => GearTrait::None,
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum EnchantType {
    AbsorbHealth,
    AbsorbMagicka,
    AbsorbStamina,
    BefouledWeapon,
    Beserker,
    ChargedWeapon,
    DamageShield,
    DiseaseResistance,
    FieryWeapon,
    FrozenWeapon,
    Health,
    HealthRegen,
    IncreaseBashDamage,
    IncreasePhysicalDamage,
    IncreasePotionEffectiveness,
    IncreaseSpellDamage,
    Magicka,
    MagickaRegen,
    OblivionDamage,
    PoisonedWeapon,
    PrismaticDefense,
    PrismaticOnslaught,
    ReduceArmor,
    ReduceBlockAndBash,
    ReduceFeatCost,
    ReducePotionCooldown,
    ReducePower,
    ReduceSpellCost,
    Stamina,
    StaminaRegen,
    Invalid,
    None,
}

pub fn match_enchant_type(string: &str) -> EnchantType {
    match string {
        "INCREASE_SPELL_DAMAGE" => EnchantType::IncreaseSpellDamage,
        "INCREASE_PHYSICAL_DAMAGE" => EnchantType::IncreasePhysicalDamage,
        "STAMINA_REGEN" => EnchantType::StaminaRegen,
        "MAGICKA_REGEN" => EnchantType::MagickaRegen,
        "HEALTH_REGEN" => EnchantType::HealthRegen,
        "REDUCE_SPELL_COST" => EnchantType::ReduceSpellCost,
        "REDUCE_FEAT_COST" => EnchantType::ReduceFeatCost,
        "REDUCE_POTION_COOLDOWN" => EnchantType::ReducePotionCooldown,
        "REDUCE_BLOCK_AND_BASH" => EnchantType::ReduceBlockAndBash,
        "INCREASE_BASH_DAMAGE" => EnchantType::IncreaseBashDamage,
        "DISEASE_RESISTANT" => EnchantType::DiseaseResistance,
        "INCREASE_POTION_EFFECTIVENESS" => EnchantType::IncreasePotionEffectiveness,
        
        "ABSORB_STAMINA" => EnchantType::AbsorbStamina,
        "ABSORB_MAGICKA" => EnchantType::AbsorbMagicka,
        "ABSORB_HEALTH" => EnchantType::AbsorbHealth,
        "CHARGED_WEAPON" => EnchantType::ChargedWeapon,
        "BEFOULED_WEAPON" => EnchantType::BefouledWeapon,
        "FROZEN_WEAPON" => EnchantType::FrozenWeapon,
        "POISONED_WEAPON" => EnchantType::PoisonedWeapon,
        "FIERY_WEAPON" => EnchantType::FieryWeapon,
        "DAMAGE_SHIELD" => EnchantType::DamageShield,
        "BERSERKER" => EnchantType::Beserker,
        "PRISMATIC_ONSLAUGHT" => EnchantType::PrismaticOnslaught,
        "REDUCE_ARMOR" => EnchantType::ReduceArmor,
        "REDUCE_POWER" => EnchantType::ReducePower,
        "DAMAGE_HEALTH" => EnchantType::OblivionDamage,

        "STAMINA" => EnchantType::Stamina,
        "MAGICKA" => EnchantType::Magicka,
        "HEALTH" => EnchantType::Health,
        "PRISMATIC_DEFENSE" => EnchantType::PrismaticDefense,
        "INVALID" => EnchantType::Invalid,
        _ => EnchantType::None,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct GearEnchant {
    pub enchant_type: EnchantType,
    pub is_cp: bool,
    pub enchant_level: u8,
    pub enchant_quality: GearQuality,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GearPiece {
    pub slot: GearSlot,
    pub item_id: u32,
    pub is_cp: bool,
    pub level: u8,
    pub gear_trait: GearTrait,
    pub quality: GearQuality,
    pub set_id: u16,
    pub enchant: Option<GearEnchant>,
}

pub fn veteran_level_to_cp(level: u8, is_cp: bool) -> u8 {
    if is_cp {
        (level.saturating_mul(10)).min(u8::MAX)
    } else {
        level
    }
}

pub fn maximum_item_level() -> u8 {
    160
}

pub fn is_appropriate_level(level: u8, is_cp: bool) -> bool {
    let level = veteran_level_to_cp(level, is_cp);
    level > 0 && level <= maximum_item_level()
}

pub fn is_maximum_item_level(level: u8, is_cp: bool) -> bool {
    veteran_level_to_cp(level, is_cp) == maximum_item_level()
}

#[cfg(test)]
mod tests {
    use crate::unit::blank_unit_state;

    use super::*;

    #[test]
    fn test_insert_gear_piece() {
        let mut player = Player {
            unit_id: 1,
            is_local_player: true,
            player_per_session_id: 1,
            class_id: Class::Dragonknight,
            race_id: Race::Breton,
            name: "Test Player".to_string(),
            display_name: "Test".to_string(),
            character_id: 12345,
            level: 10,
            champion_points: 0,
            is_grouped_with_local_player: false,
            unit_state: blank_unit_state(),
            effects: vec![],
            gear: empty_loadout(),
            primary_abilities: vec![],
            backup_abilities: vec![],
        };

        let gear_piece = GearPiece {
            slot: GearSlot::Head,
            item_id: 42,
            is_cp: false,
            level: 50,
            gear_trait: GearTrait::Sturdy,
            quality: GearQuality::Legendary,
            set_id: 100,
            enchant: None,
        };

        player.insert_gear_piece(gear_piece.clone());

        assert_eq!(player.gear.head, gear_piece);
    }

    #[test]
    fn test_match_class() {
        assert_eq!(match_class("1"), Class::Dragonknight);
        assert_eq!(match_class("117"), Class::Arcanist);
        assert_eq!(match_class("7"), Class::None);
    }

    #[test]
    fn test_match_race() {
        assert_eq!(match_race("5"), Race::Nord);
        assert_eq!(match_race("10"), Race::Imperial);
        assert_eq!(match_race("11"), Race::None);
    }

    #[test]
    fn test_match_gear_slot() {
        assert_eq!(match_gear_slot("HEAD"), GearSlot::Head);
        assert_eq!(match_gear_slot("BACKUP_OFF"), GearSlot::OffHandBackup);
        assert_eq!(match_gear_slot("INVALID"), GearSlot::None);
    }

    #[test]
    fn test_match_gear_quality() {
        assert_eq!(match_gear_quality("LEGENDARY"), GearQuality::Legendary);
        assert_eq!(match_gear_quality("NONE"), GearQuality::None);
    }

    #[test]
    fn test_match_gear_trait() {
        assert_eq!(match_gear_trait("ARMOR_STURDY"), GearTrait::Sturdy);
        assert_eq!(match_gear_trait("JEWELRY_BLOODTHIRSTY"), GearTrait::Bloodthirsty);
        assert_eq!(match_gear_trait("UNKNOWN"), GearTrait::None);
    }

    #[test]
    fn test_veteran_level_to_cp() {
        assert_eq!(veteran_level_to_cp(10, true), 100);
        assert_eq!(veteran_level_to_cp(10, false), 10);
    }

    #[test]
    fn test_is_appropriate_level() {
        assert_eq!(is_appropriate_level(50, false), true);
        assert_eq!(is_appropriate_level(0, false), false);
        assert_eq!(is_appropriate_level(200, true), false);
    }

    #[test]
    fn test_is_maximum_item_level() {
        assert!(is_maximum_item_level(16, true));
        assert!(!is_maximum_item_level(150, false));
    }

    #[test]
    fn test_empty_loadout() {
        let loadout = empty_loadout();
        assert_eq!(loadout.head.slot, GearSlot::None);
        assert_eq!(loadout.main_hand.item_id, 0);
    }
}