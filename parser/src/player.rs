use std::fmt;

use crate::{effect::Ability, unit::UnitState};
use esosim_data::item_type::ItemQuality;
use esosim_models::player::{EnchantType, GearPiece, GearSlot, GearTrait, Loadout};

#[derive(Debug, PartialEq)]
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

impl Player {
    pub fn insert_gear_piece(&mut self, slot: &GearSlot, gear_piece: GearPiece) {
        self.gear.set_gear_piece(slot, gear_piece);
    }
}

pub fn empty_loadout() -> Loadout {
    Loadout::default()
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

pub fn match_gear_slot(string: &str) -> Option<GearSlot> {
    match string {
        "HEAD" => Some(GearSlot::Head),
        "SHOULDERS" => Some(GearSlot::Shoulders),
        "CHEST" => Some(GearSlot::Chest),
        "HAND" => Some(GearSlot::Hands),
        "WAIST" => Some(GearSlot::Waist),
        "LEGS" => Some(GearSlot::Legs),
        "FEET" => Some(GearSlot::Feet),
        "NECK" => Some(GearSlot::Necklace),
        "RING1" => Some(GearSlot::Ring1),
        "RING2" => Some(GearSlot::Ring2),
        "MAIN_HAND" => Some(GearSlot::MainHand),
        "OFF_HAND" => Some(GearSlot::OffHand),
        "BACKUP_MAIN" => Some(GearSlot::MainHandBackup),
        "BACKUP_OFF" => Some(GearSlot::OffHandBackup),
        "POISON" => Some(GearSlot::Poison),
        "BACKUP_POISON" => Some(GearSlot::BackupPoison),
        _ => None,
    }
}

pub fn match_gear_quality(string: &str) -> ItemQuality {
    match string {
        "NORMAL" => ItemQuality::Normal,
        "MAGIC" => ItemQuality::Fine,
        "ARCANE" => ItemQuality::Superior,
        "ARTIFACT" => ItemQuality::Epic,
        "LEGENDARY" => ItemQuality::Legendary,
        _ => ItemQuality::Normal,
    }
}

pub fn match_gear_trait(string: &str) -> Option<GearTrait> {
    match string {
        "JEWELRY_BLOODTHIRSTY" => Some(GearTrait::Bloodthirsty),
        "JEWELRY_HARMONY" => Some(GearTrait::Harmony),
        "JEWELRY_PROTECTIVE" => Some(GearTrait::Protective),
        "JEWELRY_SWIFT" => Some(GearTrait::Swift),
        "JEWELRY_TRIUNE" => Some(GearTrait::Triune),
        "JEWELRY_INFUSED" => Some(GearTrait::Infused),
        "JEWELRY_ARCANE" => Some(GearTrait::Arcane),
        "JEWELRY_ROBUST" => Some(GearTrait::Robust),
        "JEWELRY_HEALTHY" => Some(GearTrait::Healthy),

        "ARMOR_STURDY" => Some(GearTrait::Sturdy),
        "ARMOR_IMPENETRABLE" => Some(GearTrait::Impenetrable),
        "ARMOR_REINFORCED" => Some(GearTrait::Reinforced),
        "ARMOR_WELL_FITTED" => Some(GearTrait::WellFitted),
        "ARMOR_DIVINES" => Some(GearTrait::Divines),
        "ARMOR_NIRNHONED" => Some(GearTrait::Nirnhoned),
        "ARMOR_INFUSED" => Some(GearTrait::Infused),
        "ARMOR_TRAINING" => Some(GearTrait::Training),
        "ARMOR_PROSPEROUS" => Some(GearTrait::Invigorating),

        "WEAPON_INFUSED" => Some(GearTrait::Infused),
        "WEAPON_NIRNHONED" => Some(GearTrait::Nirnhoned),
        "WEAPON_CHARGED" => Some(GearTrait::Charged),
        "WEAPON_DECISIVE" => Some(GearTrait::Decisive),
        "WEAPON_DEFENDING" => Some(GearTrait::Defending),
        "WEAPON_POWERED" => Some(GearTrait::Powered),
        "WEAPON_PRECISE" => Some(GearTrait::Precise),
        "WEAPON_SHARPENED" => Some(GearTrait::Sharpened),
        "WEAPON_TRAINING" => Some(GearTrait::Training),

        "ARMOR_INTRICATE" => Some(GearTrait::Intricate),
        "WEAPON_INTRICATE" => Some(GearTrait::Intricate),
        "JEWELRY_INTRICATE" => Some(GearTrait::Intricate),
        _ => None,
    }
}

pub fn match_enchant_type(string: &str) -> Option<EnchantType> {
    match string {
        "INCREASE_SPELL_DAMAGE" => Some(EnchantType::IncreaseSpellDamage),
        "INCREASE_PHYSICAL_DAMAGE" => Some(EnchantType::IncreasePhysicalDamage),
        "STAMINA_REGEN" => Some(EnchantType::StaminaRegen),
        "MAGICKA_REGEN" => Some(EnchantType::MagickaRegen),
        "HEALTH_REGEN" => Some(EnchantType::HealthRegen),
        "REDUCE_SPELL_COST" => Some(EnchantType::ReduceSpellCost),
        "REDUCE_FEAT_COST" => Some(EnchantType::ReduceFeatCost),
        "REDUCE_POTION_COOLDOWN" => Some(EnchantType::ReducePotionCooldown),
        "REDUCE_BLOCK_AND_BASH" => Some(EnchantType::ReduceBlockAndBash),
        "INCREASE_BASH_DAMAGE" => Some(EnchantType::IncreaseBashDamage),
        "DISEASE_RESISTANT" => Some(EnchantType::DiseaseResistance),
        "INCREASE_POTION_EFFECTIVENESS" => Some(EnchantType::IncreasePotionEffectiveness),
        "PRISMATIC_REGEN" => Some(EnchantType::PrismaticRecovery), // not implemented by ZOS yet.
        
        "ABSORB_STAMINA" => Some(EnchantType::AbsorbStamina),
        "ABSORB_MAGICKA" => Some(EnchantType::AbsorbMagicka),
        "ABSORB_HEALTH" => Some(EnchantType::AbsorbHealth),
        "CHARGED_WEAPON" => Some(EnchantType::ChargedWeapon),
        "BEFOULED_WEAPON" => Some(EnchantType::BefouledWeapon),
        "FROZEN_WEAPON" => Some(EnchantType::FrozenWeapon),
        "POISONED_WEAPON" => Some(EnchantType::PoisonedWeapon),
        "FIERY_WEAPON" => Some(EnchantType::FieryWeapon),
        "DAMAGE_SHIELD" => Some(EnchantType::DamageShield),
        "BERSERKER" => Some(EnchantType::Beserker),
        "PRISMATIC_ONSLAUGHT" => Some(EnchantType::PrismaticOnslaught),
        "REDUCE_ARMOR" => Some(EnchantType::ReduceArmor),
        "REDUCE_POWER" => Some(EnchantType::ReducePower),
        "DAMAGE_HEALTH" => Some(EnchantType::OblivionDamage),

        "STAMINA" => Some(EnchantType::Stamina),
        "MAGICKA" => Some(EnchantType::Magicka),
        "HEALTH" => Some(EnchantType::Health),
        "PRISMATIC_DEFENSE" => Some(EnchantType::PrismaticDefense),
        _ => None
    }
}

pub fn effective_level(level: u8, is_cp: bool) -> u8 {
    is_cp as u8 * 50 + level
}