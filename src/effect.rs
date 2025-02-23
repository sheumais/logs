#[derive(Debug, PartialEq)]
pub struct Effect {
    pub id: i32,
    pub name: String,
    pub icon: String,
    pub interruptible: bool,
    pub blockable: bool,
    pub stack_count: i16,
    pub effect_type: EffectType,
    pub status_effect_type: StatusEffectType,
    pub synergy: Option<i32>,
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