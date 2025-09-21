use std::{collections::{HashMap, HashSet}, fmt::{self, Display}, hash::Hash};
use parser::{effect::StatusEffectType, event::DamageType, player::Race, unit::{blank_unit_state, Reaction, UnitState}};

pub const ESO_LOGS_COM_VERSION: &'static str = "8.17.18";
pub const ESO_LOGS_PARSER_VERSION: &'static u8 = &11;
pub const LINE_COUNT_FOR_PROGRESS: usize = 25000usize;

#[derive(Default)]
pub struct ESOLogsLog {
    pub units: Vec<ESOLogsUnit>,
    pub session_id_to_units_index: HashMap<u32, usize>,
    pub owner_id_pairs_index: HashMap<(u32, usize), usize>,
    pub unit_id_to_session_id: HashMap<u32, u32>,
    pub unit_id_to_units_index: HashMap<u32, usize>,
    pub fight_units: HashMap<usize, Vec<u32>>,
    pub unit_index_during_fight: HashMap<u32, usize>,
    pub objects: HashMap<String, u32>,
    pub players: HashMap<u32, bool>,
    pub bosses: HashMap<u32, bool>,
    pub buffs: Vec<ESOLogsBuff>,
    pub buffs_hashmap: HashMap<u32, usize>,
    pub effects: Vec<ESOLogsBuffEvent>,
    pub effects_hashmap: HashMap<ESOLogsBuffEventKey, usize>,
    pub cast_id_hashmap: HashMap<u32, usize>,
    pub cast_id_target_unit_id: HashMap<u32, u32>,
    pub cast_id_source_unit_id: HashMap<u32, u32>,
    pub cast_with_cast_time: HashSet<u32>,
    pub interruption_hashmap: HashMap<ESOLogsBuffEvent, usize>,
    pub events: Vec<ESOLogsEvent>,
    pub pets: Vec<ESOLogsPetRelationship>,
    pub shields: HashMap<u32, HashMap<usize, ESOLogsBuffEventKey2>>,
    pub shield_values: HashMap<u32, u32>,
}

impl ESOLogsLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_log_reset(&mut self) {
        // self.units = Vec<ESOLogsUnit>;
        // self.session_id_to_units_index = HashMap::new();
        // self.owner_id_pairs_index = HashMap::new();
        self.unit_id_to_session_id = HashMap::new();
        self.unit_id_to_units_index = HashMap::new();
        self.fight_units = HashMap::new();
        self.unit_index_during_fight = HashMap::new();
        self.objects = HashMap::new();
        self.players = HashMap::new();
        self.bosses = HashMap::new();
        // self.buffs = Vec<ESOLogsBuff>;
        // self.buffs_hashmap = HashMap::new();
        // self.effects = Vec<ESOLogsBuffEvent>;
        self.effects_hashmap = HashMap::new();
        self.cast_id_hashmap = HashMap::new();
        self.cast_id_target_unit_id = HashMap::new();
        self.cast_id_source_unit_id = HashMap::new();
        self.cast_with_cast_time = HashSet::new();
        self.interruption_hashmap = HashMap::new();
        self.events = Vec::new();
        // self.pets = Vec<ESOLogsPetRelationship>;
        self.shields = HashMap::new();
        self.shield_values = HashMap::new();
    }

    pub fn add_unit(&mut self, unit: ESOLogsUnit) -> usize {
        let mut id = unit.unit_id;
        if let Some(pd) = &unit.player_data {
            let char_id = pd.character_id;
            if char_id != 0 {
                if let Some(existing_index) = self.units.iter().position(|u| {
                    u.player_data.as_ref().map(|p| p.character_id) == Some(char_id)
                }) {
                    if let Some(existing_unit) = self.units.get_mut(existing_index) {
                        existing_unit.unit_type = unit.unit_type;
                    }
                    self.session_id_to_units_index.insert(id, existing_index);
                    return existing_index;
                }
                let char_id_str = char_id.to_string();
                let first_9 = &char_id_str[..char_id_str.len().min(9)];
                id = first_9.parse::<u32>().unwrap_or(char_id as u32);
            }
        }

        if unit.unit_type == Reaction::Hostile && unit.owner_id != 0 {
            if let Some(existing_index) = self.units.iter().position(|u| {
                u.unit_id == unit.unit_id && u.name == unit.name
            }) {
                if let Some(existing_unit) = self.units.get_mut(existing_index) {
                    if existing_unit.owner_id != unit.owner_id {
                        existing_unit.owner_id = unit.owner_id;
                        existing_unit.unit_type = unit.unit_type;
                        // log::debug!("Setting {existing_unit} to owner id {}", unit.owner_id);
                    }
                }
                return existing_index;
            }
        }

        let owner_id = unit.owner_id;
        let session_id = self.unit_id_to_session_id.get(&owner_id).unwrap_or(&u32::MAX);
        let key = (id, *self.session_id_to_units_index.get(session_id).unwrap_or(&usize::MAX));

        if let Some(existing_index) = self.owner_id_pairs_index.get(&key) {
            if let Some(original_unit) = self.units.get_mut(*existing_index) {
                original_unit.unit_type = unit.unit_type;
            }
            if *existing_index < self.units.len() {
                return *existing_index;
            }
        }

        id = unit.unit_id;
        let index = self.units.len();
        self.units.push(unit);
        self.owner_id_pairs_index.insert(key, index);
        self.session_id_to_units_index.insert(id, index);
        index
    }

    pub fn map_unit_id_to_monster_id(&mut self, unit_id: u32, unit: &ESOLogsUnit) {
        let session_id = unit.unit_id;
        self.unit_id_to_session_id.insert(unit_id, session_id);
    }

    pub fn add_object(&mut self, object: ESOLogsUnit) -> usize {
        if self.objects.contains_key(&object.name) {
            let session_id = self.objects.get(&object.name).unwrap();
            let index = self.session_id_to_units_index.get(session_id).unwrap();
            let index2 = index.clone();
            self.session_id_to_units_index.insert(object.unit_id, *index);
            return index2;
        }
        let index = self.units.len();
        self.unit_id_to_units_index.insert(object.unit_id, index);
        self.session_id_to_units_index.insert(object.unit_id, index);
        self.objects.insert(object.name.clone(), object.unit_id);
        self.units.push(object);
        index
    }

    pub fn add_buff(&mut self, buff: ESOLogsBuff) -> bool {
        let id = buff.id;
        if self.buffs_hashmap.contains_key(&id) {
            return false;
        }
        let index = self.buffs.len();
        self.buffs.push(buff);
        self.buffs_hashmap.insert(id, index);
        true
    }

    pub fn add_buff_event(&mut self, mut buff_event: ESOLogsBuffEvent) -> usize {
        let key = ESOLogsBuffEventKey {
            source_unit_index: buff_event.source_unit_index,
            target_unit_index: buff_event.target_unit_index,
            buff_index: buff_event.buff_index,
        };
        if let Some(&idx) = self.effects_hashmap.get(&key) {
            return idx;
        }
        let index = self.effects.len();
        buff_event.unique_index = index;
        self.effects.push(buff_event);
        self.effects_hashmap.insert(key, index);
        index
    }

    pub fn unit_index(&self, unit_id: &u32) -> Option<usize> {
        // log::trace!("{:?}", self.unit_id_to_session_id);
        let res = self.unit_id_to_units_index.get(unit_id).copied();
        res
    }

    pub fn object_index(&self, object_id: String) -> Option<usize> {
        if let Some(session_id) = self.objects.get(&object_id) {
            let res = self.session_id_to_units_index.get(session_id).copied();
            res
        } else {
            None
        }
    }

    pub fn buff_index(&self, buff_id: u32) -> Option<usize> {
        let res = self.buffs_hashmap.get(&buff_id).copied();
        res
    }

    pub fn add_log_event(&mut self, event: ESOLogsEvent) {
        self.events.push(event);
    }

    pub fn get_buff_icon(&self, buff_id: u32) -> String {
        if let Some(&idx) = self.buffs_hashmap.get(&buff_id) {
            if let Some(buff) = self.buffs.get(idx) {
                return buff.icon.clone();
            }
        }
        "nil".to_string()
    }

    pub fn get_cp_for_unit(&self, unit_id: u32) -> u16 {
        if let Some(unit_index) = self.unit_id_to_units_index.get(&unit_id) {
            let cp = self.units[*unit_index].champion_points;
            return cp;
        }
        0
    }

    pub fn index_in_session(&mut self, unit_id: &u32) -> Option<usize> {
        let unit_index = self.unit_index(unit_id)?;
        // let session_id = *self.unit_id_to_session_id.get(&unit_id)?;
        if let Some(is_player) = self.players.get(&unit_id) {
            if *is_player {
                // log::trace!("Found player in index_in_session: {}, {}", unit_id, session_id);
                // if let Some(index) = self.unit_index_during_fight.get(&unit_id) {
                    // log::trace!("Player index would be: {}", index);
                // }
                return Some(0)
            }
        }
        if let Some(is_boss) = self.bosses.get(&unit_id) {
            if *is_boss {return Some(0)}
        }
        let entry = self.fight_units.entry(unit_index).or_insert_with(Vec::new);
        
        if let Some(&idx) = self.unit_index_during_fight.get(&unit_id) {return Some(idx)}

        let new_idx = entry.len();
        entry.push(*unit_id);
        self.unit_index_during_fight.insert(*unit_id, new_idx);
        Some(new_idx)
    }

    pub fn get_reaction_for_unit(&self, unit_id: u32) -> Option<Reaction> {
        if let Some(&unit_index) = self.unit_id_to_units_index.get(&unit_id) {
            return self.units.get(unit_index).map(|unit| unit.unit_type.clone());
        }
        None
    }
}

#[derive(Debug, Clone)]
pub enum ESOLogsEvent {
    Buff(ESOLogsBuffEvent),
    BuffLine(ESOLogsBuffLine),
    CastLine(ESOLogsCastLine),
    PowerEnergize(ESOLogsPowerEnergize),
    ZoneInfo(ESOLogsZoneInfo),
    PlayerInfo(ESOLogsPlayerBuild),
    MapInfo(ESOLogsMapInfo),
    EndCombat(ESOLogsCombatEvent),
    BeginCombat(ESOLogsCombatEvent),
    EndTrial(ESOLogsEndTrial),
    HealthRecovery(ESOLogsHealthRecovery),
    StackUpdate(ESOLogsBuffStacks),
    DamageShielded(ESOLogsDamageShielded),
    Interrupt(ESOLogsInterrupt),
    InterruptionEnded(ESOLogsInterruptionEnded),
    CastEnded(ESOLogsEndCast),
}

impl Display for ESOLogsEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ESOLogsEvent::Buff(e) => write!(f, "{e}"),
            ESOLogsEvent::BuffLine(e) => write!(f, "{e}"),
            ESOLogsEvent::CastLine(e) => write!(f, "{e}"),
            ESOLogsEvent::PowerEnergize(e) => write!(f, "{e}"),
            ESOLogsEvent::ZoneInfo(e) => write!(f, "{e}"),
            ESOLogsEvent::PlayerInfo(e) => write!(f, "{e}"),
            ESOLogsEvent::MapInfo(e) => write!(f, "{e}"),
            ESOLogsEvent::EndCombat(e) => write!(f, "{e}"),
            ESOLogsEvent::BeginCombat(e) => write!(f, "{e}"),
            ESOLogsEvent::EndTrial(e) => write!(f, "{e}"),
            ESOLogsEvent::HealthRecovery(e) => write!(f, "{e}"),
            ESOLogsEvent::StackUpdate(e) => write!(f, "{e}"),
            ESOLogsEvent::DamageShielded(e) => write!(f, "{e}"),
            ESOLogsEvent::Interrupt(e) => write!(f, "{e}"),
            ESOLogsEvent::InterruptionEnded(e) => write!(f, "{e}"),
            ESOLogsEvent::CastEnded(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsPlayerSpecificData {
    pub username: String,
    pub character_id: u64,
    pub is_logging_player: bool
}

#[derive(Debug, Clone)]
pub struct ESOLogsUnit {
    pub name: String,
    pub player_data: Option<ESOLogsPlayerSpecificData>,
    pub unit_type: Reaction,
    pub unit_id: u32,
    pub class: u8,
    pub server_string: String,
    pub race: Race,
    pub icon: Option<String>, // nil for players & objects, default to death_recap_melee_basic
    pub champion_points: u16,
    pub owner_id: u32,
}

impl Display for ESOLogsUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reaction = match self.unit_type {
                Reaction::Hostile => 2,
                Reaction::Friendly => 2,
                Reaction::NpcAlly => 3,
                Reaction::PlayerAlly => 1,
                Reaction::Neutral => 2,
                _ => 4,
            };
        if let Some(player_data) = &self.player_data {
            let name = if self.name.is_empty() { "nil" } else { &self.name };
            let username = if player_data.username.is_empty() { "nil" } else { &player_data.username };
            let character_id = if player_data.character_id == 0 { "nil".to_string() } else { player_data.character_id.to_string() };
            let is_logging_player = if player_data.is_logging_player { "T" } else { "F" };
            let icon = "nil";
            write!(f, "{}^{}^{}^{}|{}|{}|{}|{}|{}|{}|{}", 
            name, username, character_id, is_logging_player, reaction, self.unit_id, self.class, self.server_string, self.race, icon, self.champion_points)
        } else {
            write!(f, "{}|{}|{}|{}|{}|{}|{}|{}",
            self.name, reaction, self.unit_id, self.class, self.server_string, self.race, self.icon.as_ref().map_or("nil", |v| v), self.champion_points)
        }
    }
}

#[derive(Clone, Debug)]
pub struct ESOLogsBuff {
    pub name: String,
    pub damage_type: DamageType,
    pub status_type: StatusEffectType,
    pub id: u32,
    pub icon: String,
    pub caused_by_id: u32, // this can be itself, or another id (skills have their own id here). if none, then = 0
    pub interruptible_blockable: u8 // interruptible * 2 + blockable * 1 = number
}

impl Display for ESOLogsBuff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut damage_type = match self.damage_type {
            DamageType::Physical => 1,
            DamageType::Bleed => 2,
            DamageType::Fire => 4,
            DamageType::Cold => 16,
            DamageType::Oblivion => 32,
            DamageType::Magic => 64,
            DamageType::Disease => 256,
            DamageType::Shock => 512,
            DamageType::Heal | DamageType::Poison => 8,
            _ => 2,
        };
        if damage_type == 2 && self.status_type == StatusEffectType::Magic {damage_type = 64}
        write!(f, "{}|{}|{}|{}|{}|{}", self.name, damage_type, self.id, self.icon, self.caused_by_id, self.interruptible_blockable)
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct ESOLogsBuffEventKey2 {
    pub source_unit_index: usize,
    pub source_unit_id: u32,
    pub target_unit_index: usize,
    pub target_unit_id: u32,
    pub buff_index: usize,
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub struct ESOLogsBuffEventKey {
    pub source_unit_index: usize,
    pub target_unit_index: usize,
    pub buff_index: usize,
}


#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct ESOLogsBuffEvent {
    pub unique_index: usize,
    pub source_unit_index: usize,
    pub target_unit_index: usize,
    pub buff_index: usize,
}

impl Display for ESOLogsBuffEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}|{}", self.source_unit_index.wrapping_add(1), self.target_unit_index.wrapping_add(1), self.buff_index.wrapping_add(1))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ESOLogsLineType {
    Damage = 1,
    DotTick = 2,
    Heal = 3,
    HotTick = 4,
    BuffGainedAlly = 5,
    BuffStacksUpdatedAlly = 6,
    BuffFadedAlly = 7,
    StacksUpdatedSelf = 8,
    ShieldEvent = 9, // ??
    BuffGainedEnemy = 10,
    BuffStacksUpdatedEnemy = 11,
    BuffFadedEnemy = 12,
    CastWithCastTime = 15,
    Cast = 16,
    Death = 19,
    Resurrect = 22,
    PowerEnergize = 26,
    Interrupted = 27,
    InterruptionRemoved = 28,
    DamageShielded = 38,
    ZoneInfo = 41,
    PlayerInfo = 44,
    MapInfo = 51,
    BeginCombat = 52,
    EndCombat = 53,
    EndTrial = 55,
}

impl Display for ESOLogsLineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsBuffLine {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub buff_event: ESOLogsBuffEvent,
    pub unit_instance_id: (usize, usize),
    pub source_allegiance: u8,
    pub target_allegiance: u8,
    pub source_cast_index: Option<usize>, // A(number), index of the cast in the cast table that caused this buff change
    pub source_shield: u32,
    pub target_shield: u32,
}

impl Display for ESOLogsBuffLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (id0, id1) = self.unit_instance_id;
        let unit_instance_str = if id0 == 0 && id1 == 0 {
            format!("{}", self.buff_event.unique_index.wrapping_add(1))
        } else if id1 == 0 {
            format!("{}.{}", self.buff_event.unique_index.wrapping_add(1), id0)
        } else {
            format!("{}.{}.{}", self.buff_event.unique_index.wrapping_add(1), id0, id1)
        };
        if self.source_cast_index.is_some() {
            if (self.target_shield != 0 || self.target_shield == 0 && self.source_shield != 0) && self.target_shield != self.source_shield {
                if self.source_shield != 0 {
                    return write!(f, "{}|{}|{}|{}|{}|A{}|{}|{}", self.timestamp, self.line_type, unit_instance_str, self.source_allegiance, self.target_allegiance, self.source_cast_index.unwrap().wrapping_add(1), self.source_shield, self.target_shield);
                }
                return write!(f, "{}|{}|{}|{}|{}|A{}|{}", self.timestamp, self.line_type, unit_instance_str, self.source_allegiance, self.target_allegiance, self.source_cast_index.unwrap().wrapping_add(1), self.target_shield);
            }
            return write!(f, "{}|{}|{}|{}|{}|A{}", self.timestamp, self.line_type, unit_instance_str, self.source_allegiance, self.target_allegiance, self.source_cast_index.unwrap().wrapping_add(1));
        } else {
            write!(f, "{}|{}|{}|{}|{}", self.timestamp, self.line_type, unit_instance_str, self.source_allegiance, self.target_allegiance)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsBuffStacks {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub buff_event: ESOLogsBuffEvent,
    pub unit_instance_id: (usize, usize),
    pub source_allegiance: u8,
    pub target_allegiance: u8,
    pub stacks: u16,
}

impl Display for ESOLogsBuffStacks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (id0, id1) = self.unit_instance_id;
        let unit_instance_str = if id0 == 0 && id1 == 0 {
            format!("{}", self.buff_event.unique_index.wrapping_add(1))
        } else if id1 == 0 {
            format!("{}.{}", self.buff_event.unique_index.wrapping_add(1), id0)
        } else {
            format!("{}.{}.{}", self.buff_event.unique_index.wrapping_add(1), id0, id1)
        };
        write!(f, "{}|{}|{}|{}|{}|{}", self.timestamp, self.line_type, unit_instance_str, self.source_allegiance, self.target_allegiance, self.stacks)
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsUnitState {
    pub unit_state: UnitState,
    pub champion_points: u16, // fucking champion points. why the fuck ?????
}

impl Display for ESOLogsUnitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let map_x_int = (self.unit_state.map_x * 10_000.0).round() as i32;
            let map_y_int = (10_000f32 - (self.unit_state.map_y * 10_000.0)).round() as i32;
            let heading_int = (self.unit_state.heading * 100.0).round() as i32;
            write!(f, "{}/{}|{}/{}|{}/{}|{}/{}|{}/{}|{}|{}|{}|{}|{}",
            self.unit_state.health,
            self.unit_state.max_health,
            self.unit_state.magicka,
            self.unit_state.max_magicka,
            self.unit_state.stamina,
            self.unit_state.max_stamina,
            self.unit_state.ultimate,
            self.unit_state.max_ultimate,
            self.unit_state.werewolf,
            self.unit_state.werewolf_max,
            self.unit_state.shield,
            self.champion_points,
            map_x_int,
            map_y_int,
            heading_int
        )
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsCastData {
    pub critical: u8, // 1 = no, 2 = yes critical, 0 = ??
    pub hit_value: u32,
    pub overflow: u32,
    pub blocked: bool,
    pub override_magic_number: Option<u8>,
    pub replace_hitvalue_overflow: bool, 
}

impl Display for ESOLogsCastData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(n) = self.override_magic_number {
            write!(f, "{}", n)
        } else {
            if self.hit_value == 0 && self.overflow > 0 {
                write!(f, "{}|{}|{}", self.critical, if self.replace_hitvalue_overflow { self.hit_value } else { self.overflow }, self.overflow)
            } else if self.hit_value > 0 && self.overflow == 0 {
                if self.blocked == false {
                    write!(f, "{}|{}", self.critical, self.hit_value)
                } else {
                    write!(f, "{}|{}|{}|{}", 4, self.hit_value, 0, 1)
                }
            } else {
                write!(f, "{}|{}|{}", self.critical, self.hit_value + self.overflow, self.overflow)
            }
        }

    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsCastBase {
    pub source_allegiance: u8,
    pub target_allegiance: u8,
    pub cast_id_origin: u32,
    pub source_unit_state: ESOLogsUnitState,
    pub target_unit_state: ESOLogsUnitState,
}

impl Display for ESOLogsCastBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cast_str = if self.cast_id_origin == 0 {
            format!("")
        } else {
            format!("|C{}", self.cast_id_origin)
        };
        if self.target_unit_state.unit_state == blank_unit_state() {
            return write!(f, "{}|{}{}|S{}", self.source_allegiance, self.target_allegiance, cast_str, self.source_unit_state)
        }
        write!(f, "{}|{}{}|S{}|T{}", self.source_allegiance, self.target_allegiance, cast_str, self.source_unit_state, self.target_unit_state)
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsCastLine {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub buff_event: ESOLogsBuffEvent,
    pub unit_instance_id: (usize, usize),
    pub cast: ESOLogsCastBase,
    pub cast_information: Option<ESOLogsCastData>,
}

impl Display for ESOLogsCastLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (id0, id1) = self.unit_instance_id;
        let unit_instance_str = if id0 == 0 && id1 == 0 {
            format!("{}", self.buff_event.unique_index.wrapping_add(1))
        } else if id1 == 0 {
            format!("{}.{}", self.buff_event.unique_index.wrapping_add(1), id0)
        } else {
            format!("{}.{}.{}", self.buff_event.unique_index.wrapping_add(1), id0, id1)
        };
        if let Some(cast_info) = &self.cast_information {
            if self.cast.target_unit_state.unit_state.health == 0 {
                write!(f, "{}|{}|{}|{}|{}|{}|0|0|{}", self.timestamp, self.line_type, unit_instance_str, self.cast, cast_info.critical, cast_info.hit_value + cast_info.overflow, cast_info.overflow)
            } else {
                write!(f, "{}|{}|{}|{}|{}", self.timestamp, self.line_type, unit_instance_str, self.cast, cast_info)
            }
        } else {
            write!(f, "{}|{}|{}|{}", self.timestamp, self.line_type, unit_instance_str, self.cast)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ESOLogsResourceType {
    Health = 4,
    Magicka = 0,
    Stamina = 1,
    Ultimate = 2,
}

impl Display for ESOLogsResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsPowerEnergize {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub buff_event: ESOLogsBuffEvent,
    pub cast: ESOLogsCastBase,
    pub hit_value: i32,
    pub overflow: u32,
    pub resource_type: ESOLogsResourceType,
}

impl Display for ESOLogsPowerEnergize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_of_that_resource = match self.resource_type {
            ESOLogsResourceType::Health => self.cast.target_unit_state.unit_state.max_health, // this should never happen. an energize of health is called a heal!
            ESOLogsResourceType::Magicka => self.cast.target_unit_state.unit_state.max_magicka,
            ESOLogsResourceType::Stamina => self.cast.target_unit_state.unit_state.max_stamina,
            ESOLogsResourceType::Ultimate => self.cast.target_unit_state.unit_state.max_ultimate,
        };
        write!(f, "{}|{}|{}|{}|{}|{}|{}|{}", self.timestamp, self.line_type, self.buff_event.unique_index.wrapping_add(1), self.cast, self.hit_value, self.overflow, self.resource_type, max_of_that_resource)
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsZoneInfo {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub zone_id: u16,
    pub zone_name: String,
    pub zone_difficulty: u8, // 0 none, 1 = normal, 2 = veteran
}

impl Display for ESOLogsZoneInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}|{}|{}|{}", self.timestamp, self.line_type, self.zone_id, self.zone_name, self.zone_difficulty)
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsMapInfo {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub map_id: u16,
    pub map_name: String,
    pub map_image_url: String,
}

impl Display for ESOLogsMapInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}|{}|{}|{}", self.timestamp, self.line_type, self.map_id, self.map_name, self.map_image_url)
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsPlayerBuild {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub unit_index: usize,
    pub permanent_buffs: String,
    pub buff_stacks: String,
    pub gear: Vec<String>,
    pub primary_abilities: String,
    pub backup_abilities: String,
}

impl Display for ESOLogsPlayerBuild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let gear = self.gear.join("],[");
        write!(f, "{}|{}|{}|[{}],[{}],[[{}]],[{}],[{}]", self.timestamp, self.line_type, self.unit_index.wrapping_add(1), self.permanent_buffs, self.buff_stacks, gear, self.primary_abilities, self.backup_abilities)
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsCombatEvent {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
}

impl Display for ESOLogsCombatEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|{}|", self.timestamp, self.line_type)
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsEndTrial {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub trial_id: u8,
    pub duration: u64,
    pub success: u8, // 1 = success, 0 = fail
    pub final_score: u32,
}

impl Display for ESOLogsEndTrial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|{}|{}|{}|{}|{}", self.timestamp, self.line_type, self.trial_id, self.duration, self.success, self.final_score)
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsHealthRecovery {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub buff_event: ESOLogsBuffEvent,
    pub effective_regen: u32,
    pub unit_state: ESOLogsUnitState,
}

impl Display for ESOLogsHealthRecovery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|{}|{}|16|16|S{}|T{}|1|{}", self.timestamp, self.line_type, self.buff_event.unique_index.wrapping_add(1), self.unit_state, self.unit_state, self.effective_regen)
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsPetRelationship {
    pub owner_index: usize,
    pub pet: ESOLogsPet,
}

impl Display for ESOLogsPetRelationship {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|{}", self.pet, self.owner_index.wrapping_add(1))
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsPet {
    pub pet_type_index: usize,
}

impl Display for ESOLogsPet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pet_type_index.wrapping_add(1))
    }
}

// 5314073|38|17895.1.1|64|64|6|0|16|0|451|609
// timestamp | linetype | unit_instance_string for original shield | source allegiance | target allegiance | damage source instance id | damage source allegiance | 0 | hit_value | source_cast_index
#[derive(Debug, Clone)]
pub struct ESOLogsDamageShielded { // purely for healing purposes. we copy the overflow value to a damage event
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub buff_event: ESOLogsBuffEvent,
    pub damage_source_allegiance: u8,
    pub shield_source_allegiance: u8,
    pub shield_recipient_allegiance: u8,
    pub unit_instance_id: (usize, usize),
    pub orig_shield_instance_ids: (usize, usize),
    pub hit_value: u32,
    pub source_ability_cast_index: usize,
}

impl Display for ESOLogsDamageShielded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (id0, id1) = self.unit_instance_id;
        let unit_instance_str = if id0 == 0 && id1 == 0 {
            format!("{}", self.buff_event.unique_index.wrapping_add(1))
        } else if id1 == 0 {
            format!("{}.{}", self.buff_event.unique_index.wrapping_add(1), id0)
        } else {
            format!("{}.{}.{}", self.buff_event.unique_index.wrapping_add(1), id0, id1)
        };
        write!(f, "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}", self.timestamp, self.line_type, unit_instance_str, self.shield_source_allegiance, self.shield_recipient_allegiance, self.buff_event.source_unit_index.wrapping_add(1), self.orig_shield_instance_ids.0, self.damage_source_allegiance, 0, self.hit_value, self.source_ability_cast_index.wrapping_add(1))
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsInterrupt {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub buff_event: ESOLogsBuffEvent,
    pub unit_instance_id: (usize, usize),
    pub source_allegiance: u8,
    pub target_allegiance: u8,
    pub interrupted_ability_index: usize,
}

impl Display for ESOLogsInterrupt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (id0, id1) = self.unit_instance_id;
        let unit_instance_str = if id0 == 0 && id1 == 0 {
            format!("{}", self.buff_event.unique_index.wrapping_add(1))
        } else if id1 == 0 {
            format!("{}.{}", self.buff_event.unique_index.wrapping_add(1), id0)
        } else {
            format!("{}.{}.{}", self.buff_event.unique_index.wrapping_add(1), id0, id1)
        };
        write!(f, "{}|{}|{}|{}|{}|{}", self.timestamp, self.line_type, unit_instance_str, self.source_allegiance, self.target_allegiance, self.interrupted_ability_index.wrapping_add(1))
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsInterruptionEnded {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub buff_event: ESOLogsBuffEvent,
    pub unit_instance_id: (usize, usize),
    pub source_allegiance: u8,
    pub target_allegiance: u8,
    pub interruption_index: usize,
    pub magic_number: u8,
}

impl Display for ESOLogsInterruptionEnded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (id0, id1) = self.unit_instance_id;
        let unit_instance_str = if id0 == 0 && id1 == 0 {
            format!("{}", self.buff_event.unique_index.wrapping_add(1))
        } else if id1 == 0 {
            format!("{}.{}", self.buff_event.unique_index.wrapping_add(1), id0)
        } else {
            format!("{}.{}.{}", self.buff_event.unique_index.wrapping_add(1), id0, id1)
        };
        write!(f, "{}|{}|{}|{}|{}|{}|{}", self.timestamp, self.line_type, unit_instance_str, self.source_allegiance, self.target_allegiance, self.interruption_index.wrapping_add(1), self.magic_number)
    }
}

#[derive(Debug, Clone)]
pub struct ESOLogsEndCast {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub buff_event: ESOLogsBuffEvent,
    pub unit_instance_id: (usize, usize),
    pub source_allegiance: u8,
    pub target_allegiance: u8
}

impl Display for ESOLogsEndCast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (id0, id1) = self.unit_instance_id;
        let unit_instance_str = if id0 == 0 && id1 == 0 {
            format!("{}", self.buff_event.unique_index.wrapping_add(1))
        } else if id1 == 0 {
            format!("{}.{}", self.buff_event.unique_index.wrapping_add(1), id0)
        } else {
            format!("{}.{}.{}", self.buff_event.unique_index.wrapping_add(1), id0, id1)
        };
        write!(f, "{}|{}|{}|{}|{}", self.timestamp, self.line_type, unit_instance_str, self.source_allegiance, self.target_allegiance)
    }
}
