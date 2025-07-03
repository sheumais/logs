use std::{collections::HashMap, fmt::{self, Display}, hash::Hash};
use parser::{effect::StatusEffectType, event::DamageType, player::Race, unit::{blank_unit_state, Reaction, UnitState}};
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct ESOLogsLog {
    pub units: Vec<ESOLogsUnit>,
    pub session_id_to_units_index: HashMap<u32, usize>,
    pub unit_id_to_session_id: HashMap<u32, u32>,
    pub session_units: HashMap<u32, Vec<u32>>,
    pub unit_index_in_session: HashMap<u32, usize>,
    pub objects: HashMap<String, u32>,
    pub buffs: Vec<ESOLogsBuff>,
    pub buffs_hashmap: HashMap<u32, usize>,
    pub effects: Vec<ESOLogsBuffEvent>,
    pub effects_hashmap: HashMap<ESOLogsBuffEventKey, usize>,
    pub events: Vec<ESOLogsEvent>
}

impl ESOLogsLog {
    pub fn new() -> Self {
        Self::default()
    }


    /// Insert a combat unit. Returns `true` when successful and `false` if the
    /// session‑scoped unit already exists.
    pub fn add_unit(&mut self, unit: ESOLogsUnit) -> bool {
        // println!("add_unit - attempting to add unit: session_id = {} (name = {})", unit.unit_id, unit.name);
        let id = unit.unit_id;
        if self.session_id_to_units_index.contains_key(&id) {
            // println!("add_unit - unit with session_id {} already present; skipping", id);
            return false;
        }
        let index = self.units.len() + 1; // 1‑based because raw logs use that convention
        self.units.push(unit);
        self.session_id_to_units_index.insert(id, index);
        // println!("add_unit - unit inserted at overall index {}", index);
        true
    }

    /// Map a *combat* unit_id that appears inside events to its parent `session_id`.
    pub fn map_unit_id_to_monster_id(&mut self, unit_id: u32, session_id: u32) -> bool {
        // println!("map_unit_id_to_monster_id - unit_id {} ↦ session_id {}", unit_id, session_id);
        if self.unit_id_to_session_id.contains_key(&unit_id) {
            // println!("map_unit_id_to_monster_id - mapping already exists; aborting");
            return false;
        }
        self.unit_id_to_session_id.insert(unit_id, session_id);

        let pos = self.session_units.entry(session_id).or_default().len();
        self.session_units.get_mut(&session_id).unwrap().push(unit_id);
        self.unit_index_in_session.insert(unit_id, pos);
        // println!("map_unit_id_to_monster_id - mapping added; index within session = {}", pos);
        true
    }

    /// Add an *object* (e.g. boss mechanic entity) that lives outside the unit tables.
    pub fn add_object(&mut self, object: ESOLogsUnit) -> bool {
        // println!("add_object - attempting to add object: {} (unit_id = {})", object.name, object.unit_id);
        if self.objects.contains_key(&object.name) {
            // println!("add_object - object '{}' already registered; skipping", object.name);
            return false;
        }
        let index = self.units.len() + 1;
        self.session_id_to_units_index.insert(object.unit_id, index);
        self.objects.insert(object.name.clone(), object.unit_id);
        self.units.push(object);
        // println!("add_object - object inserted at overall index {}", index);
        true
    }

    /// Register a new buff definition.
    pub fn add_buff(&mut self, buff: ESOLogsBuff) -> bool {
        // println!("add_buff - id {} ({})", buff.id, buff.name);
        let id = buff.id;
        if self.buffs_hashmap.contains_key(&id) {
            // println!("add_buff - buff {} already exists; skipping", id);
            return false;
        }
        let index = self.buffs.len() + 1;
        self.buffs.push(buff);
        self.buffs_hashmap.insert(id, index);
        // println!("add_buff - buff stored at index {}", index);
        true
    }

    /// Deduplicate and/or insert a buff application/removal event.
    /// Returns the unique index of the resulting `ESOLogsBuffEvent` in `self.effects`.
    pub fn add_buff_event(&mut self, mut buff_event: ESOLogsBuffEvent) -> usize {
        let key = ESOLogsBuffEventKey {
            source_unit_index: buff_event.source_unit_index,
            target_unit_index: buff_event.target_unit_index,
            buff_index: buff_event.buff_index,
        };
        // println!("add_buff_event - looking for existing key {:?}", key);
        if let Some(&idx) = self.effects_hashmap.get(&key) {
            // println!("add_buff_event - event already present at index {}", idx);
            return idx;
        }
        let index = self.effects.len() + 1;
        buff_event.unique_index = index;
        self.effects.push(buff_event);
        self.effects_hashmap.insert(key, index);
        // println!("add_buff_event - new buff event stored at index {}", index);
        index
    }

    /// Translate a combat `unit_id` to the canonical 1‑based `unit_index` used by ESO‑Logs.
    pub fn unit_index(&self, unit_id: u32) -> Option<usize> {
        // println!("unit_index - resolving unit_id {}", unit_id);
        if let Some(&session_id) = self.unit_id_to_session_id.get(&unit_id) {
            let res = self.session_id_to_units_index.get(&session_id).copied();
            // println!("unit_index - resolved to {:?}", res);
            res
        } else {
            // println!("unit_index - unit_id {} not found", unit_id);
            None
        }
    }

    /// Lookup the index for a given `buff_id`.
    pub fn buff_index(&self, buff_id: u32) -> Option<usize> {
        // println!("buff_index - resolving buff_id {}", buff_id);
        let res = self.buffs_hashmap.get(&buff_id).copied();
        // println!("buff_index - resolved to {:?}", res);
        res
    }

    /// Append a generic combat `event` to the flat list.
    pub fn add_log_event(&mut self, event: ESOLogsEvent) {
        self.events.push(event);
    }

    /// Fetch the *icon path* for a given `buff_id`.
    pub fn get_buff_icon(&self, buff_id: u32) -> String {
        // println!("get_buff_icon - buff_id {}", buff_id);
        if let Some(&idx) = self.buffs_hashmap.get(&buff_id) {
            if let Some(buff) = self.buffs.get(idx - 1) {
                // println!("get_buff_icon - found icon '{}'", buff.icon);
                return buff.icon.clone();
            }
        }
        // println!("get_buff_icon - buff_id {} unknown; returning 'nil'", buff_id);
        "nil".to_string()
    }

    /// Return champion points for a given unit, or `0` when unknown.
    pub fn get_cp_for_unit(&self, unit_id: u32) -> u16 {
        // println!("get_cp_for_unit - unit_id {}", unit_id);
        if let Some(&session_id) = self.unit_id_to_session_id.get(&unit_id) {
            if let Some(unit_index) = self.session_id_to_units_index.get(&session_id) {
                let cp = self.units[*unit_index - 1].champion_points;
                // println!("get_cp_for_unit - CP = {}", cp);
                return cp;
            }
        }
        // println!("get_cp_for_unit - CP unknown for unit_id {}", unit_id);
        0
    }

    /// Position of `unit_id` within its session's unit array.
    pub fn index_in_session(&self, unit_id: u32) -> Option<usize> {
        // println!("index_in_session - unit_id {}", unit_id);
        let res = self.unit_index_in_session.get(&unit_id).copied();
        // println!("index_in_session - resolved to {:?}", res);
        res
    }

    /// Convenience wrapper around `session_units` that hides the mutable `Vec`.
    pub fn units_for_session(&self, session_id: u32) -> Option<&[u32]> {
        // println!("units_for_session - session_id {}", session_id);
        let res = self.session_units.get(&session_id).map(|v| v.as_slice());
        // println!("units_for_session - found {} units", res.map_or(0, |s| s.len()));
        res
    }

    /// Get the Reaction of the session unit corresponding to a given unit_id.
    pub fn get_reaction_for_unit(&self, unit_id: u32) -> Option<Reaction> {
        if let Some(&session_id) = self.unit_id_to_session_id.get(&unit_id) {
            if let Some(&unit_index) = self.session_id_to_units_index.get(&session_id) {
                return self.units.get(unit_index.saturating_sub(1)).map(|unit| unit.unit_type.clone());
            }
        }
        None
    }
}

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
        }
    }
}

pub struct ESOLogsPlayerSpecificData {
    pub username: String,
    pub character_id: u64,
    pub is_logging_player: bool
}

pub struct ESOLogsUnit {
    pub name: String,
    pub player_data: Option<ESOLogsPlayerSpecificData>,
    pub unit_type: Reaction,
    pub unit_id: u32,
    pub class: u8,
    pub server_string: String,
    pub race: Race,
    pub icon: Option<String>, // nil for players, non-trivial to compute. default to death_recap_melee_basic
    pub champion_points: u16
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

// 612,ABILITY_INFO,45509,"Penetrating Magic","/esoui/art/icons/ability_weapon_008.dds",T,T          3
// 612,ABILITY_INFO,30959,"Ancient Knowledge","/esoui/art/icons/ability_weapon_003.dds",F,T          1
// 1913,ABILITY_INFO,61506,"Echoing Vigor","/esoui/art/icons/ability_ava_echoing_vigor.dds",F,F      0
#[derive(Clone)]
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
            DamageType::Heal | DamageType::Poison => 8, // heal
            _ => 2,
        };
        if damage_type == 2 && self.status_type == StatusEffectType::Magic {damage_type = 64}
        write!(f, "{}|{}|{}|{}|{}|{}", self.name, damage_type, self.id, self.icon, self.caused_by_id, self.interruptible_blockable)
    }
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub struct ESOLogsBuffEventKey {
    source_unit_index: u16,
    target_unit_index: u16,
    buff_index: u32,
}

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct ESOLogsBuffEvent {
    pub unique_index: usize,
    pub source_unit_index: u16,
    pub target_unit_index: u16,
    pub buff_index: u32,
}

impl Display for ESOLogsBuffEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}|{}", self.source_unit_index, self.target_unit_index, self.buff_index)
    }
}

#[derive(Copy, Clone)]
pub enum ESOLogsLineType {
    Damage = 1,
    DotTick = 2,
    Heal = 3,
    HotTick = 4,
    BuffGainedSelf = 5,
    BuffStacksUpdated = 6, // stacks after buff table reference (52438|6|37|16|16|3)
    BuffFaded = 7,
    BuffGainedTarget = 10,
    DebuffStacksUpdated = 11,
    FadedOnOthers = 12,
    CastWithCastTime = 15,
    Cast = 16,
    Death = 19,
    PowerEnergize = 26,
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

pub struct ESOLogsBuffLine {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType, // BuffFaded or BuffGained
    pub buff_event: ESOLogsBuffEvent, // print only the index
    pub unit_instance_id: (usize, usize),
    pub source_allegiance: u8, // often 16, sometimes 32, maybe some other stuff
    pub target_allegiance: u8, // always 16?
    pub source_cast_index: Option<usize>, // A(number), index of the cast in the cast table that caused this buff change
    pub source_shield: u32, // shield amount
    pub target_shield: u32,
}

impl Display for ESOLogsBuffLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (id0, id1) = self.unit_instance_id;
        let unit_instance_str = if id0 == 0 && id1 == 0 {
            format!("{}", self.buff_event.unique_index)
        } else if id1 == 0 {
            format!("{}.{}", self.buff_event.unique_index, id0)
        } else {
            format!("{}.{}.{}", self.buff_event.unique_index, id0, id1)
        };
        if self.source_cast_index.is_some() {
            if (self.target_shield != 0 || self.target_shield == 0 && self.source_shield != 0) && self.target_shield != self.source_shield {
                if self.source_shield != 0 {
                    return write!(f, "{}|{}|{}|{}|{}|A{}|{}|{}", self.timestamp, self.line_type, unit_instance_str, self.source_allegiance, self.target_allegiance, self.source_cast_index.unwrap(), self.source_shield, self.target_shield);
                }
                return write!(f, "{}|{}|{}|{}|{}|A{}|{}", self.timestamp, self.line_type, unit_instance_str, self.source_allegiance, self.target_allegiance, self.source_cast_index.unwrap(), self.target_shield);
            }
            return write!(f, "{}|{}|{}|{}|{}|A{}", self.timestamp, self.line_type, unit_instance_str, self.source_allegiance, self.target_allegiance, self.source_cast_index.unwrap());
        } else {
            write!(f, "{}|{}|{}|{}|{}", self.timestamp, self.line_type, unit_instance_str, self.source_allegiance, self.target_allegiance)
        }
    }
}

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
            format!("{}", self.buff_event.unique_index)
        } else if id1 == 0 {
            format!("{}.{}", self.buff_event.unique_index, id0)
        } else {
            format!("{}.{}.{}", self.buff_event.unique_index, id0, id1)
        };
        write!(f, "{}|{}|{}|{}|{}|{}", self.timestamp, self.line_type, unit_instance_str, self.source_allegiance, self.target_allegiance, self.stacks)
    }
}

pub struct ESOLogsUnitState {
    pub unit_state: UnitState,
    pub champion_points: u16, // fucking champion points. why the fuck ?????
}

impl Display for ESOLogsUnitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let map_x_int = (self.unit_state.map_x * 10_000.0).round() as u32;
            let map_y_int = (10_000f32 - (self.unit_state.map_y * 10_000.0)).round() as u32;
            let heading_int = (self.unit_state.heading * 100.0).round() as u32;
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

pub struct ESOLogsCastData {
    pub critical: u8, // 1 = no, 2 = yes critical, 0 = ??
    pub hit_value: u32, // set equal to overflow if = 0
    pub overflow: u32, 
}

impl Display for ESOLogsCastData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.hit_value == 0 && self.overflow > 0 {
            write!(f, "{}|{}|{}", self.critical, self.overflow, self.overflow)
        } else if self.hit_value > 0 && self.overflow == 0 {
            write!(f, "{}|{}", self.critical, self.hit_value)
        } else {
            write!(f, "{}|{}|{}", self.critical, self.hit_value, self.overflow)
        }
    }
}

pub struct ESOLogsCastBase {
    pub source_allegiance: u8, // source allegiance (16 = friendly, 32 = ally?, 64 = enemy)
    pub target_allegiance: u8, // target allegiance
    pub cast_id_origin: u32,
    pub source_unit_state: ESOLogsUnitState,
    pub target_unit_state: ESOLogsUnitState,
}

impl Display for ESOLogsCastBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.target_unit_state.unit_state == blank_unit_state() {
            return write!(f, "{}|{}|C{}|S{}", self.source_allegiance, self.target_allegiance, self.cast_id_origin, self.source_unit_state)
        }
        write!(f, "{}|{}|C{}|S{}|T{}", self.source_allegiance, self.target_allegiance, self.cast_id_origin, self.source_unit_state, self.target_unit_state)
    }
}


pub struct ESOLogsCastLine {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType, // CastOnOthers or CastOnSelf if source == target
    pub buff_event: ESOLogsBuffEvent,
    pub unit_instance_id: (usize, usize),
    pub cast: ESOLogsCastBase,
    pub cast_information: Option<ESOLogsCastData>,
}

impl Display for ESOLogsCastLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (id0, id1) = self.unit_instance_id;
        let unit_instance_str = if id0 == 0 && id1 == 0 {
            format!("{}", self.buff_event.unique_index)
        } else if id1 == 0 {
            format!("{}.{}", self.buff_event.unique_index, id0)
        } else {
            format!("{}.{}.{}", self.buff_event.unique_index, id0, id1)
        };
        if let Some(cast_info) = &self.cast_information {
            write!(f, "{}|{}|{}|{}|{}", self.timestamp, self.line_type, unit_instance_str, self.cast, cast_info)
        } else {
            write!(f, "{}|{}|{}|{}", self.timestamp, self.line_type, unit_instance_str, self.cast)
        }
    }
}

#[derive(Copy, Clone)]
pub enum ESOLogsResourceType {
    Health = 4,
    Magicka = 0,
    Stamina = 1,
    Ultimate = 8,
}

impl Display for ESOLogsResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

pub struct ESOLogsPowerEnergize {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType, // PowerEnergize
    pub buff_event: ESOLogsBuffEvent,
    pub cast: ESOLogsCastBase,
    pub hit_value: u32,
    pub overflow: u32,
    pub resource_type: ESOLogsResourceType,
}

impl Display for ESOLogsPowerEnergize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_of_that_resource = match self.resource_type {
            ESOLogsResourceType::Health => self.cast.source_unit_state.unit_state.max_health,
            ESOLogsResourceType::Magicka => self.cast.source_unit_state.unit_state.max_magicka,
            ESOLogsResourceType::Stamina => self.cast.source_unit_state.unit_state.max_stamina,
            ESOLogsResourceType::Ultimate => self.cast.source_unit_state.unit_state.max_ultimate,
        };
        write!(f, "{}|{}|{}|{}|{}|{}|{}|{}", self.timestamp, self.line_type, self.buff_event.unique_index, self.cast, self.hit_value, self.overflow, self.resource_type, max_of_that_resource)
    }
}


pub struct ESOLogsZoneInfo {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType, // ZoneInfo
    pub zone_id: u16,
    pub zone_name: String,
    pub zone_difficulty: u8, // 0 none, 1 = normal, 2 = veteran
}

impl Display for ESOLogsZoneInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}|{}|{}|{}", self.timestamp, self.line_type, self.zone_id, self.zone_name, self.zone_difficulty)
    }
}


pub struct ESOLogsMapInfo {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType, // MapInfo
    pub map_id: u16,
    pub map_name: String,
    pub map_image_url: String,
}

impl Display for ESOLogsMapInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}|{}|{}|{}", self.timestamp, self.line_type, self.map_id, self.map_name, self.map_image_url)
    }
}

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
        write!(f, "{}|{}|{}|[{}],[{}],[[{}]],[{}],[{}]", self.timestamp, self.line_type, self.unit_index, self.permanent_buffs, self.buff_stacks, gear, self.primary_abilities, self.backup_abilities)
    }
}

pub struct ESOLogsCombatEvent {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
}

impl Display for ESOLogsCombatEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|{}|", self.timestamp, self.line_type)
    }
}

pub struct ESOLogsEndTrial {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub trial_id: u8,
    pub duration: u64,
    pub success: u8, // 1 = success, 0 = fail
    pub final_score: u32,
    // pub vitality_bonus: u16, not used
}

impl Display for ESOLogsEndTrial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|{}|{}|{}|{}|{}", self.timestamp, self.line_type, self.trial_id, self.duration, self.success, self.final_score)
    }
}

pub struct ESOLogsHealthRecovery {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType,
    pub buff_event: ESOLogsBuffEvent,
    pub effective_regen: u32,
    pub unit_state: ESOLogsUnitState,
}

impl Display for ESOLogsHealthRecovery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|{}|{}|16|16|S{}|T{}|1|{}", self.timestamp, self.line_type, self.buff_event.unique_index, self.unit_state, self.unit_state, self.effective_regen)
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub user: UserInfo,
    pub enabled_features: EnabledFeatures,
    #[serde(rename = "guildSelectItems")]
    pub guild_select_items: Vec<GuildSelectInfo>,
    #[serde(rename = "reportVisibilitySelectItems")]
    pub report_visibility_select_items: Vec<LabelValue>,
    #[serde(rename = "regionOrServerSelectItems")]
    pub region_or_server_select_items: Vec<ValueLabel>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: u32,
    #[serde(rename = "userName")]
    pub username: String,
    #[serde(rename = "emailAddress")]
    pub email_address: Option<String>,
    #[serde(rename = "isAdmin")]
    pub is_admin: bool,
    pub guilds: Vec<GuildInfo>,
    #[serde(default)]
    pub characters: Vec<CharacterInfo>,
    pub thumbnail: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnabledFeatures {
    #[serde(rename = "noAds")]
    pub no_ads: bool,
    #[serde(rename = "realTimeLiveLogging")]
    pub real_time_live_logging: bool,
    pub meters: bool,
    #[serde(rename = "liveFightData")]
    pub live_fight_data: bool,
    #[serde(rename = "tooltipAddon")]
    pub tooltip_addon: bool,
    #[serde(rename = "tooltipAddonTierTwoData")]
    pub tooltip_addon_tier_two_data: bool,
    #[serde(rename = "autoLog")]
    pub auto_log: bool,
    #[serde(rename = "metersLiveParse")]
    pub meters_live_parse: bool,
    #[serde(rename = "metersRaceTheGhost")]
    pub meters_race_the_ghost: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GuildInfo {
    pub id: u16,
    pub name: String,
    pub rank: u8,
    pub guild_logo: GuildLogo,
    pub faction: u8,
    #[serde(rename = "isRecruit")]
    pub is_recruit: bool,
    #[serde(rename = "isOfficer")]
    pub is_officer: bool,
    #[serde(rename = "isGuildMaster")]
    pub is_guild_master: bool,
    pub server: Server,
    pub region: Region,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GuildSelectInfo {
    pub value: i32, // -1 for personal logs
    pub label: String,
    pub logo: GuildLogo,
    #[serde(rename = "cssClassName")]
    pub css_class_name: String,
    #[serde(rename = "regionId")]
    pub region_id: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GuildLogo {
    pub url: String,
    #[serde(rename = "isCustom")]
    pub is_custom: bool,
    #[serde(rename = "fallbackUrl")]
    pub fallback_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CharacterInfo {
    pub id: u32,
    pub name: String,
    #[serde(rename = "cssClassName")]
    pub class_name: String,
    pub thumbnail: String,
    pub server: Server,
    pub region: Region,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: u8,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    pub id: u8,
    pub name: String,
    #[serde(rename = "shortName")]
    pub short_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LabelValue {
    pub label: String,
    pub value: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ValueLabel {
    pub label: String,
    pub value: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EncounterReportCode {
    pub code: String
}