use serde::{Deserialize, Serialize};
use crate::{player::Player, set::get_item_type_from_hashmap, unit::UnitState};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Ability {
    pub id: u32,
    pub name: String,
    pub icon: String,
    pub interruptible: bool,
    pub blockable: bool,
    pub scribing: Option<Vec<String>>
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub ability: Ability,
    pub stack_count: u16,
    pub effect_type: EffectType,
    pub status_effect_type: StatusEffectType,
    pub synergy: Option<u32>,
}

#[derive(Debug, PartialEq)]
pub struct EffectEvent {
    pub time: u64,
    pub change_type: EffectChangeType,
    pub stack_count: u16,
    pub cast_track_id: u32,
    pub ability_id: u32,
    pub source_unit_state: UnitState,
    pub target_unit_state: UnitState,
    pub player_initiated_remove_cast_track_id: bool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EffectChangeType {
    Faded,
    Gained,
    Updated,
    None,
}

pub fn parse_effect_change_type(string: &str) -> EffectChangeType {
    match string {
        "FADED" => EffectChangeType::Faded,
        "GAINED" => EffectChangeType::Gained,
        "UPDATED" => EffectChangeType::Updated,
        _ => EffectChangeType::None
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum EffectType {
    Buff,
    Debuff,
    None,
}

pub fn parse_effect_type(string: &str) -> EffectType {
    match string {
        "BUFF" => EffectType::Buff,
        "DEBUFF" => EffectType::Debuff,
        _ => EffectType::None,
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum StatusEffectType {
    Magic,
    None,
}

pub fn parse_status_effect_type(string: &str) -> StatusEffectType {
    match string {
        "MAGIC" => StatusEffectType::Magic,
        _ => StatusEffectType::None,
    }
}

pub const ZEN_DEBUFF_ID: &'static u32 = &126597;

pub fn is_zen_dot(effect_id: u32) -> bool {
    match effect_id {
            // class abilities
        36947 => true, // debilitate
        35336 => true, // lotus fan
        36960 => true, // crippling grasp

        21731 => true, // vampire's bane
        21732 => true, // reflective light

        101944 => true, // growing swarm
        101904 => true, // fetcher infection
        130140 => true, // cutting dive

        20326 => true, // volatile armour
        31898 => true, // burning talons
        31103 => true, // noxious breath
        31104 => true, // engulfing flames
        44369 => true, // venomous claw
        44373 => true, // burning embers

        118618 => true, // pure agony synergy
        143944 => true, // ghostly embrace (2nd circle)

        182989 => true, // fulminating rune
        185840 => true, // rune of displacement

            // weapon abilities
        204009 => true, // tri focus (fire staff)
        38747 => true, // carve
        62712 => true, // frost reach
        62682 => true, // flame reach
        62745 => true, // shock reach
        38703 => true, // acid spray
        44549 => true, // poison injection 
        85261 => true, // toxic barrage
        44545 => true, // venom arrow
        85182 => true, // thrive in chaos
        // rend ultimate
        38848 => true, // rending slashes
        38845 => true, // blood craze


            // world abilities
        137259 => true, // exhilarating drain (vamp)
        // drain vigor (vamp)
        126895 => true, // soul splitting trap
        126897 => true, // consuming trap
        // soul assault
        // shatter soul


            // gear sets
        76667 => true, // alkosh (line-breaker)
        97743 => true, // pillar of nirn
        172671 => true, // whorl of the depths
        107203 => true, // arms of relequen

            // guild abilities
        40468 => true, // scalding rune
        40385 => true, // barbed trap
        40375 => true, // lightweight barbed trap
        126374 => true, // degeneration
        126371 => true, // structured entropy
        62314 => true, // dawnbreaker of smiting
        62310 => true, // flawless dawnbreaker

            // other
        18084 => true, // burning
        21929 => true, // poisoned
        148801 => true, // hemorrhaging
        41838 => true, // radiate synergy
        113627 => true, // virulent shot (brp bow)
        79025 => true, // ravage health 3.5s
        219720 => true, // travelling knife lingering torment
        _ => false
    }
}

pub enum SummonablePets {
    // Sorcerer
    UnstableFamiliar = 23304,
    VolatileFamiliar = 23316,
    UnstableClannfear = 23319,
    GreaterStormAtronach = 23492,
    ChargedAtronach = 23495,
    StormAtronach = 23634,
    WingedTwilight = 24613,
    TwilightMatriach = 24639,
    TwilightTormenter = 24636,
    // Nightblade
    Shade = 33211,
    DarkShade = 35434,
    ShadowImage = 35441,
    // Undaunted
    // TrappingWebs =,
    // ShadowSilk =,
    // TanglingWebs =,
    // Werewolf
    // Pack leader ultimate ???
    // Warden
    ViolentGuardian = 85982,
    EternalGuardian = 85986,
    WildGuardian = 85990,
    // Necromancer
    SpiritMender = 115710,
    IntensiveMender = 118840,
    SpiritGuardian = 118912,
    SacrificialBones = 114860,
    GraveLordsSacrifice = 117749,
    BlightedBlastbones = 117690,
    SkeletalMage = 114317,
    SkeletalArcher = 118680,
    SkeletonArcanist = 118726,
    // Arcanist
    VitalizingGlyphic = 183709,
    GlyphicOfTheTides = 193794,
    ResonatingGlyphic = 193558, // Hostile!
}

fn update_blockade_ability(ability: &mut Ability, item_type: &crate::set::ItemType) {
    match item_type {
        crate::set::ItemType::FrostStaff => {
            ability.icon = "/esoui/art/icons/ability_destructionstaff_002b.dds".to_string();
            ability.name = "Blockade of Frost".to_string();
        },
        crate::set::ItemType::LightningStaff => {
            ability.icon = "/esoui/art/icons/ability_destructionstaff_003_b.dds".to_string();
            ability.name = "Blockade of Storms".to_string();
        },
        crate::set::ItemType::FireStaff => {
            ability.icon = "/esoui/art/icons/ability_destructionstaff_004_b.dds".to_string();
            ability.name = "Fiery Blockade".to_string();
        },
        _ => {},
    }
}

pub fn destruction_staff_skill_convert(player: &mut Player) {
    let item_type = get_item_type_from_hashmap(player.gear.main_hand.item_id);
    for ability in &mut player.primary_abilities {
        if ability.id == 39011 {
            update_blockade_ability(ability, &item_type);
        }
    }
    let backup_item_type = get_item_type_from_hashmap(player.gear.main_hand_backup.item_id);
    for ability in &mut player.backup_abilities {
        if ability.id == 39011 {
            update_blockade_ability(ability, &backup_item_type);
        }
    }
}