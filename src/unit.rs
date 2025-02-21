use std::fmt;

pub fn is_true(value: &str) -> bool {
    value == "T"
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum UnitType {
    Player,
    Monster,
    None,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ClassId {
    DragonKnight,
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
        "1" => ClassId::DragonKnight,
        "2" => ClassId::Sorcerer,
        "3" => ClassId::Nightblade,
        "4" => ClassId::Warden,
        "5" => ClassId::Necromancer,
        "6" => ClassId::Templar,
        "117" => ClassId::Arcanist,
        _ => ClassId::None,
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
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
        "3" => RaceId::Orc,
        "4" => RaceId::DarkElf,
        "5" => RaceId::Nord,
        "7" => RaceId::HighElf,
        "9" => RaceId::Khajiit,
        _ => RaceId::None,
    }
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

pub struct Unit {
    pub unit_id: i32,
    pub unit_type: UnitType,
    pub is_local_player: bool,
    pub player_per_session_id: i32,
    pub monster_id: i32,
    pub is_boss: bool,
    pub class_id: ClassId,
    pub race_id: RaceId,
    pub name: String,
    pub display_name: String,
    pub character_id: i128,
    pub level: i8,
    pub champion_points: i16,
    pub owner_unit_id: i32,
    pub reaction: Reaction,
    pub is_grouped_with_local_player: bool
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ unit_id: {}, unit_type: {:?}, player_per_session_id: {}, monster_id: {}, is_boss: {}, class_id: {:?}, race_id: {:?}, name: {}, display_name: {}, character_id: {}, level: {}, champion_points: {}, owner_unit_id: {}, reaction: {:?} }}",
            self.unit_id,
            self.unit_type,
            self.player_per_session_id,
            self.monster_id,
            self.is_boss,
            self.class_id,
            self.race_id,
            self.name,
            self.display_name,
            self.character_id,
            self.level,
            self.champion_points,
            self.owner_unit_id,
            self.reaction
        )
    }
}