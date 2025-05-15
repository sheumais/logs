#[derive(Debug, PartialEq)]
pub enum UnitType {
    Monster,
    Object,
}


#[derive(Debug, PartialEq)]
pub enum Reaction {
    PlayerAlly,
    NpcAlly,
    Hostile,
    Neutral,
    None,
}

pub fn match_reaction(string: &str) -> Reaction {
    match string {
        "PLAYER_ALLY" => Reaction::PlayerAlly,
        "NPC_ALLY" => Reaction::NpcAlly,
        "HOSTILE" => Reaction::Hostile,
        "NEUTRAL" => Reaction::Neutral,
        _ => Reaction::None,
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct UnitState {
    pub unit_id: u32,
    pub health: u32,
    pub max_health: u32,
    pub magicka: u32,
    pub max_magicka: u32,
    pub stamina: u32,
    pub max_stamina: u32,
    pub ultimate: u32,
    pub max_ultimate: u32,
    pub werewolf: u32,
    pub werewolf_max: u32,
    pub shield: u32,
    pub map_x: f32,
    pub map_y: f32,
    /// Units are radians
    pub heading: f32,
}

pub fn blank_unit_state() -> UnitState {
    UnitState {
        unit_id: 0,
        health: 0,
        max_health: 0,
        magicka: 0,
        max_magicka: 0,
        stamina: 0,
        max_stamina: 0,
        ultimate: 0,
        max_ultimate: 0,
        werewolf: 0,
        werewolf_max: 0,
        shield: 0,
        map_x: 0.0,
        map_y: 0.0,
        heading: 0.0,
    }
}

#[derive(Debug, PartialEq)]
pub struct Unit {
    pub unit_id: u32,
    pub unit_type: UnitType,
    pub monster_id: u32,
    pub is_boss: bool,
    pub name: String,
    pub level: u8,
    pub champion_points: u16,
    pub owner_unit_id: u32,
    pub reaction: Reaction,
    pub unit_state: UnitState,
    pub effects: Vec<u32>,
}