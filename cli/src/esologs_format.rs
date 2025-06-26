// Normal parser
// Collect each unit for unit table
// Collect each definition for definition table
// Collect each effect_gained event for table
// Collect each effect_faded event for table
// Collect each begin_cast event for table

/// File 1
// 15|1| <-- note extra | for some reason. no other line has this
// #Units
// [Unit array]
// #Effects
// [Effect array]
// #Gain Events
// [Effect gain array]
// #??
// 24|6?

/// File 2
// 15|1
// #Events
// [Effect/Cast array]

use std::{collections::HashMap, fmt::{self, Display}, hash::Hash};
use parser::{event::DamageType, player::Race, unit::{Reaction, UnitState}};

#[derive(Default)]
pub struct ESOLogsLog {
    pub units: Vec<ESOLogsUnit>,
    pub units_hashmap: HashMap<u32, usize>,
    pub unit_id_to_session_id: HashMap<u32, u32>,
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

    pub fn add_unit(&mut self, unit: ESOLogsUnit) -> bool {
        let id = unit.unit_id;
        if self.units_hashmap.contains_key(&id) {
            return false;
        }
        let index = self.units.len()+1;
        self.units.push(unit);
        self.units_hashmap.insert(id, index);
        true
    }

    pub fn map_unit_id_to_monster_id(&mut self, unit_id: u32, monster_id: u32) -> bool {
        if self.unit_id_to_session_id.contains_key(&unit_id) {
            return false;
        }
        self.unit_id_to_session_id.insert(unit_id, monster_id);
        return true
    }

    pub fn add_object(&mut self, object: ESOLogsUnit) -> bool {
        if self.objects.contains_key(&object.name) {
            return false;
        }
        let index = self.units.len()+1;
        self.units_hashmap.insert(object.unit_id, index);
        self.objects.insert(object.name.clone(), object.unit_id);
        self.units.push(object);
        true
    }

    pub fn add_buff(&mut self, buff: ESOLogsBuff) -> bool {
        let id = buff.id;
        if self.buffs_hashmap.contains_key(&id) {
            return false;
        }
        let index = self.buffs.len()+1;
        self.buffs.push(buff);
        self.buffs_hashmap.insert(id, index);
        true
    }

    pub fn add_buff_event(&mut self, mut buff_event: ESOLogsBuffEvent) -> usize {
        let key = ESOLogsBuffEventKey {
            source_unit_index: buff_event.source_unit_index,
            target_unit_index: buff_event.target_unit_index,
            buff_index: buff_event.buff_index
        };
        if let Some(&idx) = self.effects_hashmap.get(&key) {
            return idx;
        }
        let index = self.effects.len()+1;
        buff_event.unique_index = index;
        self.effects.push(buff_event);
        self.effects_hashmap.insert(key, index);
        index
    }

    pub fn unit_index(&self, unit_id: u32) -> Option<usize> {
        if let Some(&session_id) = self.unit_id_to_session_id.get(&unit_id) {
            self.units_hashmap.get(&session_id).copied()
        } else {
            None
        }
    }

    pub fn buff_index(&self, buff_id: u32) -> Option<usize> {
        if let Some(&id) = self.buffs_hashmap.get(&buff_id) {
            Some(id.clone())
        } else {
            None
        }
    }

    pub fn add_log_event(&mut self, event: ESOLogsEvent) {
        self.events.push(event);
    }

    pub fn get_buff_icon(&mut self, buff_id: u32) -> String {
        if let Some(&idx) = self.buffs_hashmap.get(&buff_id) {
            if let Some(buff) = self.buffs.get(idx - 1) {
                return buff.icon.clone();
            }
        }
        "nil".to_string()
    }
}

pub enum ESOLogsEvent {
    Buff(ESOLogsBuffEvent),
    BuffLine(ESOLogsBuffLine),
    CastLine(ESOLogsCastLine),
    PowerEnergize(ESOLogsPowerEnergize),
    ZoneInfo(ESOLogsZoneInfo),
    MapInfo(ESOLogsMapInfo),
}

impl Display for ESOLogsEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ESOLogsEvent::Buff(e)           => write!(f, "{e}"),
            ESOLogsEvent::BuffLine(e)       => write!(f, "{e}"),
            ESOLogsEvent::CastLine(e)       => write!(f, "{e}"),
            ESOLogsEvent::PowerEnergize(e)  => write!(f, "{e}"),
            ESOLogsEvent::ZoneInfo(e)       => write!(f, "{e}"),
            ESOLogsEvent::MapInfo(e)        => write!(f, "{e}"),
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
        if self.player_data.is_some() {
            let player_data = self.player_data.as_ref().unwrap();
            let is_logging_player = if player_data.is_logging_player == true {"T"} else {"F"};
            write!(f, "{}^{}^{}^{}|{}|{}|{}|{}|{}|{}|{}", self.name, player_data.username, player_data.character_id, is_logging_player, reaction, self.unit_id, self.class, self.server_string, self.race, "nil", self.champion_points)
        } else {
            write!(f, "{}|{}|{}|{}|{}|{}|{}|{}", self.name, reaction, self.unit_id, self.class, self.server_string, self.race, self.icon.as_ref().map_or("nil", |v| v), self.champion_points)
        }
    }
}

// 612,ABILITY_INFO,45509,"Penetrating Magic","/esoui/art/icons/ability_weapon_008.dds",T,T          3
// 612,ABILITY_INFO,30959,"Ancient Knowledge","/esoui/art/icons/ability_weapon_003.dds",F,T          1
// 1913,ABILITY_INFO,61506,"Echoing Vigor","/esoui/art/icons/ability_ava_echoing_vigor.dds",F,F      0

pub struct ESOLogsBuff {
    pub name: String,
    pub damage_type: DamageType,
    pub id: u32,
    pub icon: String,
    pub caused_by_id: u32, // this can be itself, or another id (skills have their own id here). if none, then = 0
    pub interruptible_blockable: u8 // interruptible * 2 + blockable * 1 = number
}

impl Display for ESOLogsBuff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let damage_type = match self.damage_type {
            DamageType::Physical => 1,
            DamageType::Bleed => 2,
            DamageType::Fire => 4,
            DamageType::Poison => 8,
            DamageType::Cold => 16,
            DamageType::Magic => 64,
            DamageType::Disease => 256,
            DamageType::Shock => 512,
            _ => 2,
        };
        write!(f, "{}|{}|{}|{}|{}|{}", self.name, damage_type, self.id, self.icon, self.caused_by_id, self.interruptible_blockable)
    }
}

#[derive(Eq, Hash, PartialEq)]
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
    CastOnOthers = 4,
    BuffFaded = 5,
    BuffGained = 7,
    CastOnSelf = 16,
    PowerEnergize = 26,
    ZoneInfo = 41,
    MapInfo = 51,
    EndCombat = 53,
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
    pub magic_number_1: u8, // often 16, sometimes 32, maybe some other stuff
    pub magic_number_2: u8, // always 16?
    pub magic_entry: Option<u16>, // A(number) or smthn. idk why this appears sometimes
    pub magic_entry_2: Option<u16>, // Sometimes after the A()| there will be another number
}

impl Display for ESOLogsBuffLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.magic_entry.is_some() {
            if self.magic_entry_2.is_some() {
                return write!(f, "{}|{}|{}|{}|{}|A{}|{}", self.timestamp, self.line_type, self.buff_event.unique_index, self.magic_number_1, self.magic_number_2, self.magic_entry.unwrap(), self.magic_entry_2.unwrap());
            }
            return write!(f, "{}|{}|{}|{}|{}|A{}", self.timestamp, self.line_type, self.buff_event.unique_index, self.magic_number_1, self.magic_number_2, self.magic_entry.unwrap());
        } else {
            write!(f, "{}|{}|{}|{}|{}", self.timestamp, self.line_type, self.buff_event.unique_index, self.magic_number_1, self.magic_number_2)
        }
    }
}

pub struct ESOLogsUnitState {
    pub unit_state: UnitState,
    pub magic_index: u16, // no idea.
}

impl Display for ESOLogsUnitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let map_x_int = (self.unit_state.map_x * 10_000.0).round() as u32;
            let map_y_int = (self.unit_state.map_y * 10_000.0).round() as u32;
            let heading_int = (self.unit_state.heading * 100.0).round() as u32;
            write!(f, "{}/{}|{}/{}|{}/{}|{}/{}|{}/{}|{}|{}|{}|{}",
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
        } else {
            write!(f, "{}|{}|{}", self.critical, self.hit_value, self.overflow)
        }
    }
}

pub struct ESOLogsCastBase {
    pub magic_number_1: u8, // often 16, sometimes 32, maybe some other stuff
    pub magic_number_2: u8, // always 16?
    pub cast_id_origin: u32,
    pub source_unit_state: ESOLogsUnitState,
    pub target_unit_state: ESOLogsUnitState,
}

impl Display for ESOLogsCastBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}|C{}|S{}|T{}", self.magic_number_1, self.magic_number_2, self.cast_id_origin, self.source_unit_state, self.target_unit_state)
    }
}


pub struct ESOLogsCastLine {
    pub timestamp: u64,
    pub line_type: ESOLogsLineType, // CastOnOthers or CastOnSelf if source == target
    pub buff_event: ESOLogsBuffEvent,
    pub cast: ESOLogsCastBase,
    pub cast_information: Option<ESOLogsCastData>,
}

impl Display for ESOLogsCastLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.cast_information.is_some() {
            write!(f, "{}|{}|{}|{}|{}", self.timestamp, self.line_type, self.buff_event.unique_index, self.cast, self.cast_information.as_ref().unwrap())
        } else {
            write!(f, "{}|{}|{}|{}", self.timestamp, self.line_type, self.buff_event.unique_index, self.cast)
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

//4282,COMBAT_EVENT,POWER_ENERGIZE,GENERIC,1,224,0,20091228,131489,36,22394/22394,25108/32968,9601/12770,500/500,1000/1000,0,0.6116,0.3636,4.5066,*
//4282,COMBAT_EVENT,POWER_ENERGIZE,GENERIC,4,224,0,20091228,99781,36,22394/22394,25108/32968,9825/12770,500/500,1000/1000,0,0.6116,0.3636,4.5066,*
//4270|26|85|16|16|C20091228|S22394/22394|25108/32968|9601/12770|500/500|1000/1000|0|2195|6116|6364|450|T22394/22394|25108/32968|9601/12770|500/500|1000/1000|0|2195|6116|6364|450|224|0|0|32968
//4270|26|84|16|16|C20091228|S22394/22394|25108/32968|9825/12770|500/500|1000/1000|0|2195|6116|6364|450|T22394/22394|25108/32968|9825/12770|500/500|1000/1000|0|2195|6116|6364|450|224|0|1|12770
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
    pub zone_difficulty: u8, // 2 = veteran
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