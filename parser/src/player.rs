use crate::{effect::Ability, unit::UnitState};

#[derive(Debug, PartialEq, Clone)]
pub struct Player {
    pub unit_id: u32,
    pub is_local_player: bool,
    pub player_per_session_id: u32,
    pub class_id: ClassId,
    pub race_id: RaceId,
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
        enchant: GearEnchant {
            enchant_type: EnchantType::None,
            is_cp: false,
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
pub enum ClassId {
    Dragonknight,
    Sorcerer,
    Nightblade,
    Templar,
    Warden,
    Necromancer,
    Arcanist,
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
        _ => ClassId::Nightblade,
    }
}

#[derive(Debug, PartialEq, Clone)]
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
    pub enchant: GearEnchant,
}

pub fn veteran_level_to_cp(level: u8, is_cp: bool) -> u8 {
    if is_cp {
        level * 10
    } else {
        level
    }
}

pub fn maximum_item_level() -> u8 {
    160
}

#[allow(dead_code)]
pub fn is_appropriate_level(level: u8, is_cp: bool) -> bool {
    let level = veteran_level_to_cp(level, is_cp);
    level > 0 && level <= maximum_item_level()
}

pub fn is_maximum_item_level(level: u8, is_cp: bool) -> bool {
    veteran_level_to_cp(level, is_cp) == maximum_item_level()
}