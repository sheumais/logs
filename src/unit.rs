use std::fmt;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum UnitType {
    Monster,
    Player,
    Object,
    None,
}


#[derive(Debug)]
#[derive(PartialEq)]
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

pub struct UnitState {
    pub health: i32,
    pub max_health: i32,
    pub magicka: i32,
    pub max_magicka: i32,
    pub stamina: i32,
    pub max_stamina: i32,
    pub ultimate: i32,
    pub max_ultimate: i32,
    pub werewolf: i32,
    pub werewolf_max: i32,
    pub shield: i32,
    pub map_x: f32,
    pub map_y: f32,
    pub heading: f32, // Radians
}

pub fn blank_unit_state() -> UnitState {
    UnitState {
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

pub struct Effect {
    pub id: i32,
    pub name: String,
    pub icon: String,
    pub stacks: i16,
    pub time_remaining: i32,
    pub effect_type: i32,
    pub status_effect_type: i32,
    pub grants_synergy: bool,
}

pub struct Unit {
    pub unit_id: i32,
    pub unit_type: UnitType,
    pub monster_id: i32,
    pub is_boss: bool,
    pub name: String,
    pub level: i8,
    pub champion_points: i16,
    pub owner_unit_id: i32,
    pub reaction: Reaction,
    pub unit_state: UnitState,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ unit_id: {}, unit_type: {:?}, monster_id: {}, is_boss: {}, name: {}, level: {}, champion_points: {}, owner_unit_id: {}, reaction: {:?} }}",
            self.unit_id,
            self.unit_type,
            self.monster_id,
            self.is_boss,
            self.name,
            self.level,
            self.champion_points,
            self.owner_unit_id,
            self.reaction
        )
    }
}