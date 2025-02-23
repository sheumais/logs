use crate::unit::{UnitType, UnitState};
use std::fmt;

pub struct Player {
    pub unit_id: i32,
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
    pub gear: Loadout,
    pub primary_abilities: Vec<i32>,
    pub backup_abilities: Vec<i32>,
}

#[derive(Debug, PartialEq)]
pub struct Loadout {
    pub head: GearPiece,
    pub shoulders: GearPiece,
    pub chest: GearPiece,
    pub hands: GearPiece,
    pub waist: GearPiece,
    pub legs: GearPiece,
    pub feet: GearPiece,
    pub neck: GearPiece,
    pub ring1: GearPiece,
    pub ring2: GearPiece,
    pub main_hand: GearPiece,
    pub main_hand_backup: GearPiece,
    pub off_hand: GearPiece,
    pub off_hand_backup: GearPiece,
}

impl Player {
    pub fn insert_gear_piece(&mut self, gear_piece: GearPiece) {
        let slot = gear_piece.slot.clone();
        self.gear.insert(slot, gear_piece);
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Player {{ unit_id: {}, is_local_player: {}, player_per_session_id: {}, class_id: {:?}, race_id: {:?}, name: {}, display_name: {}, character_id: {}, level: {}, champion_points: {}, is_grouped_with_local_player: {}, unit_state: {:?}, effects: {:?}, gear: {}, primary_abilities: {:?}, backup_abilities: {:?} }}",
            self.unit_id,
            self.is_local_player,
            self.player_per_session_id,
            self.class_id,
            self.race_id,
            self.name,
            self.display_name,
            self.character_id,
            self.level,
            self.champion_points,
            self.is_grouped_with_local_player,
            self.unit_state,
            self.effects,
            self.gear,
            self.primary_abilities,
            self.backup_abilities
        )
    }
}

impl fmt::Display for Loadout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Loadout {{ head: {}, shoulders: {}, chest: {}, hands: {}, waist: {}, legs: {}, feet: {}, neck: {}, ring1: {}, ring2: {}, main_hand: {}, main_hand_backup: {}, off_hand: {}, off_hand_backup: {} }}",
            self.head,
            self.shoulders,
            self.chest,
            self.hands,
            self.waist,
            self.legs,
            self.feet,
            self.neck,
            self.ring1,
            self.ring2,
            self.main_hand,
            self.main_hand_backup,
            self.off_hand,
            self.off_hand_backup
        )
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
            GearSlot::Neck => self.neck = gear_piece,
            GearSlot::Ring1 => self.ring1 = gear_piece,
            GearSlot::Ring2 => self.ring2 = gear_piece,
            GearSlot::MainHand => self.main_hand = gear_piece,
            GearSlot::MainHandBackup => self.main_hand_backup = gear_piece,
            GearSlot::OffHand => self.off_hand = gear_piece,
            GearSlot::OffHandBackup => self.off_hand_backup = gear_piece,
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
        trait_id: GearTrait::None,
        quality: GearQuality::None,
        set_id: 0,
        enchant: GearEnchant {
            enchant_type: EnchantType::None,
            is_enchant_cp: false,
            enchant_level: 0,
            enchant_quality: GearQuality::None,
        },
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
        neck: empty_gear_piece(),
        ring1: empty_gear_piece(),
        ring2: empty_gear_piece(),
        main_hand: empty_gear_piece(),
        main_hand_backup: empty_gear_piece(),
        off_hand: empty_gear_piece(),
        off_hand_backup: empty_gear_piece(),
    }
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq, Clone)]
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
    Costume,
    Poison,
    BackupOff,
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
        "NECK" => GearSlot::Neck,
        "RING1" => GearSlot::Ring1,
        "RING2" => GearSlot::Ring2,
        "MAIN_HAND" => GearSlot::MainHand,
        "OFF_HAND" => GearSlot::MainHandBackup,
        "BACKUP" => GearSlot::OffHand,
        "BACKUP_MAIN" => GearSlot::OffHandBackup,
        "COSTUME" => GearSlot::Costume,
        "BACKUP_OFF" => GearSlot::BackupOff,
        _ => GearSlot::None,
    }
}

#[derive(Debug, PartialEq)]
pub enum GearQuality {
    Normal,
    Fine,
    Superior,
    Epic,
    Legendary,
    Artifact,
    Arcane,
    None,
}

pub fn match_gear_quality(string: &str) -> GearQuality {
    match string {
        "NORMAL" => GearQuality::Normal,
        "FINE" => GearQuality::Fine,
        "SUPERIOR" => GearQuality::Superior,
        "EPIC" => GearQuality::Epic,
        "LEGENDARY" => GearQuality::Legendary,
        "ARTIFACT" => GearQuality::Artifact,
        "ARCANE" => GearQuality::Arcane,
        _ => GearQuality::None,
    }
}

#[derive(Debug, PartialEq)]
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
    Prosperous,
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
        "ARMOR_INVIGORATING" => GearTrait::Invigorating,
        "ARMOR_DIVINES" => GearTrait::Divines,
        "ARMOR_NIRNHONED" => GearTrait::Nirnhoned,
        "ARMOR_INFUSED" => GearTrait::Infused,
        "ARMOR_TRAINING" => GearTrait::Training,
        "ARMOR_PROSPEROUS" => GearTrait::Prosperous,

        "WEAPON_INFUSED" => GearTrait::Infused,
        "WEAPON_NIRNHONED" => GearTrait::Nirnhoned,
        "WEAPON_CHARGED" => GearTrait::Charged,
        "WEAPON_DECISIVE" => GearTrait::Decisive,
        "WEAPON_DEFENDING" => GearTrait::Defending,
        "WEAPON_POWERED" => GearTrait::Powered,
        "WEAPON_PRECISE" => GearTrait::Precise,
        "WEAPON_SHARPENED" => GearTrait::Sharpened,
        "WEAPON_TRAINING" => GearTrait::Training,
        _ => GearTrait::None,
    }
}


#[derive(Debug, PartialEq)]
pub enum EnchantType {
    AbsorbHealth,
    AbsorbMagicka,
    AbsorbStamina,
    BefouledWeapon,
    Beserker,
    ChargedWeapon,
    DamageShield,
    FieryWeapon,
    FrozenWeapon,
    Health,
    HealthRegen,
    IncreasePhysicalDamage,
    IncreaseSpellDamage,
    Magicka,
    MagickaRegen,
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

        "STAMINA" => EnchantType::Stamina,
        "MAGICKA" => EnchantType::Magicka,
        "HEALTH" => EnchantType::Health,
        "PRISMATIC_DEFENSE" => EnchantType::PrismaticDefense,
        "INVALID" => EnchantType::Invalid,
        _ => EnchantType::None,
    }
}
#[derive(Debug, PartialEq)]
pub struct GearEnchant {
    pub enchant_type: EnchantType,
    pub is_enchant_cp: bool,
    pub enchant_level: i8,
    pub enchant_quality: GearQuality,
}
#[derive(Debug, PartialEq)]
pub struct GearPiece {
    pub slot: GearSlot,
    pub item_id: i32,
    pub is_cp: bool,
    pub level: i8,
    pub trait_id: GearTrait,
    pub quality: GearQuality,
    pub set_id: i32,
    pub enchant: GearEnchant,
}

impl fmt::Display for GearPiece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ slot: {:?}, item_id: {}, is_cp: {}, level: {}, trait_id: {:?}, quality: {:?}, set_id: {}, enchant: {:?} }}",
            self.slot,
            self.item_id,
            self.is_cp,
            self.level,
            self.trait_id,
            self.quality,
            self.set_id,
            self.enchant
        )
    }
}

impl fmt::Display for GearEnchant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ enchant_type: {:?}, is_enchant_cp: {}, enchant_level: {}, enchant_quality: {:?} }}",
            self.enchant_type,
            self.is_enchant_cp,
            self.enchant_level,
            self.enchant_quality
        )
    }
}