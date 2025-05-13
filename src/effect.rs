use crate::unit::UnitState;

#[derive(Debug, PartialEq, Clone)]
pub struct Ability {
    pub id: u32,
    pub name: String,
    pub icon: String,
    pub interruptible: bool,
    pub blockable: bool,
    pub scribing: Option<Vec<String>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Effect {
    pub id: u32,
    pub name: String,
    pub icon: String,
    pub interruptible: bool,
    pub blockable: bool,
    pub stack_count: u16,
    pub effect_type: EffectType,
    pub status_effect_type: StatusEffectType,
    pub synergy: Option<u32>,
    pub scribing: Option<Vec<String>>
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
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

#[allow(dead_code)]
pub fn is_zen_dot(ability_id: u32, scribing: Option<Vec<String>>) -> bool {
    match ability_id {
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

        // pure agony synergy
        // ghostly embrace (2nd circle)

        182989 => true, // fulminating rune
        185840 => true, // rune of displacement

        // weapon abilities
        204009 => true, // tri focus (fire staff)
        38747 => true, // carve
        62712 => true, // frost reach
        38703 => true, // acid spray
        44549 => true, // poison injection 
        85261 => true, // toxic barrage
        44545 => true, // venom arrow
        38841 => true, // rending slashes
        85182 => true, // thrive in chaos
        // rend
        // blood craze


        // world abilities
        137259 => true, // exhilarating drain
        // drain vigor
        // soul splitting trap
        // consuming trap
        // soul assault
        // shatter soul


        // armor sets
        75753 => true, // alkosh (line-breaker)
        97743 => true, // pillar of nirn
        172671 => true, // whorl of the depths
        107203 => true, // arms of relequen

        // guild abilities
        40468 => true, // scalding rune
        40385 => true, // barbed trap
        // lightweight barbed trap
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

        _ => {
            if scribing.is_some() {
                scribing.unwrap()[1] == "Lingering Torment"
            } else {
                false
            }
        }
    }
}