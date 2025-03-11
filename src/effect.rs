use crate::unit::UnitState;

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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