use std::{collections::HashMap, error::Error, fs::File, io::{BufRead, BufReader, BufWriter}, path::Path};
use std::io::Write;
use parser::{effect::{self, StatusEffectType}, event::{self, parse_cast_end_reason, CastEndReason, DamageType, EventResult}, parse::{self}, player::{Class, Race}, unit::{self, blank_unit_state, Reaction, UnitState}, EventType, UnitAddedEventType};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};
use std::fs;

use crate::{esologs_format::*, log_edit::{handle_line, CustomLogData}};

pub fn event_timestamp(e: &ESOLogsEvent) -> Option<u64> {
    match e {
        ESOLogsEvent::BuffLine(e) => Some(e.timestamp),
        ESOLogsEvent::CastLine(e) => Some(e.timestamp),
        ESOLogsEvent::PowerEnergize(e) => Some(e.timestamp),
        ESOLogsEvent::ZoneInfo(e) => Some(e.timestamp),
        ESOLogsEvent::PlayerInfo(e) => Some(e.timestamp),
        ESOLogsEvent::MapInfo(e) => Some(e.timestamp),
        ESOLogsEvent::EndCombat(e) => Some(e.timestamp),
        ESOLogsEvent::BeginCombat(e) => Some(e.timestamp),
        ESOLogsEvent::EndTrial(e) => Some(e.timestamp),
        ESOLogsEvent::HealthRecovery(e) => Some(e.timestamp),
        ESOLogsEvent::StackUpdate(e) => Some(e.timestamp),
        ESOLogsEvent::DamageShielded(e) => Some(e.timestamp),
        ESOLogsEvent::Interrupt(e) => Some(e.timestamp),
        ESOLogsEvent::InterruptionEnded(e) => Some(e.timestamp),
        ESOLogsEvent::CastEnded(e) => Some(e.timestamp),
        _ => None,
    }
}

fn set_event_timestamp(e: &mut ESOLogsEvent, timestamp: u64) {
    match e {
        ESOLogsEvent::BuffLine(inner) => inner.timestamp = timestamp,
        ESOLogsEvent::CastLine(inner) => inner.timestamp = timestamp,
        ESOLogsEvent::PowerEnergize(inner) => inner.timestamp = timestamp,
        ESOLogsEvent::ZoneInfo(inner) => inner.timestamp = timestamp,
        ESOLogsEvent::PlayerInfo(inner) => inner.timestamp = timestamp,
        ESOLogsEvent::MapInfo(inner) => inner.timestamp = timestamp,
        ESOLogsEvent::EndCombat(inner) => inner.timestamp = timestamp,
        ESOLogsEvent::BeginCombat(inner) => inner.timestamp = timestamp,
        ESOLogsEvent::EndTrial(inner) => inner.timestamp = timestamp,
        ESOLogsEvent::HealthRecovery(inner) => inner.timestamp = timestamp,
        ESOLogsEvent::StackUpdate(inner) => inner.timestamp = timestamp,
        ESOLogsEvent::DamageShielded(inner) => inner.timestamp = timestamp,
        ESOLogsEvent::Interrupt(inner) => inner.timestamp = timestamp,
        ESOLogsEvent::InterruptionEnded(inner) => inner.timestamp = timestamp,
        ESOLogsEvent::CastEnded(inner) => inner.timestamp = timestamp,
        _ => {}
    }
}

fn is_damage_event(e: &ESOLogsEvent) -> bool {
    match e {
        ESOLogsEvent::CastLine(cl) => matches!(cl.line_type, ESOLogsLineType::Damage | ESOLogsLineType::DotTick),
        _ => false,
    }
}

fn get_cast_id(e: &ESOLogsEvent) -> Option<u64> {
    match e {
        ESOLogsEvent::CastLine(cl) => Some(cl.cast.cast_id_origin.into()),
        _ => None,
    }
}

fn get_target_id(e: &ESOLogsEvent) -> Option<u64> {
    match e {
        ESOLogsEvent::CastLine(cl) => Some(cl.cast.target_unit_state.unit_state.unit_id.into()),
        _ => None,
    }
}

pub struct ESOLogProcessor {
    pub eso_logs_log: ESOLogsLog,
    pub megaserver: String,
    pub timestamp_offset: u64,
    temporary_damage_buffer: u32,
    pending_death_events: Vec<(u64, ESOLogsEvent)>, // (timestamp, event)
    last_known_timestamp: u64,
    active_casts: HashMap<u32, usize>, // (unit_id, index of ability)
    last_interrupt: Option<u32>, // unit_id of last interrupted unit
    base_timestamp: Option<u64>,
    most_recent_begin_log_timestamp: Option<u64>,
}

impl ESOLogProcessor {
    pub fn new() -> Self {
        Self {
            eso_logs_log: ESOLogsLog::new(),
            megaserver: "Unknown".to_owned(),
            timestamp_offset: 0,
            temporary_damage_buffer: 0,
            pending_death_events: Vec::new(),
            last_known_timestamp: 0,
            active_casts: HashMap::new(),
            last_interrupt: None,
            base_timestamp: None,
            most_recent_begin_log_timestamp: None,
        }
    }

    pub fn reset(&mut self) {
        self.temporary_damage_buffer = 0;
        self.pending_death_events = Vec::new();
        self.last_known_timestamp = 0;
        self.active_casts = HashMap::new();
        self.last_interrupt = None;
    }

    pub fn add_unit(&mut self, unit: ESOLogsUnit) -> usize {
        return self.eso_logs_log.add_unit(unit)
    }

    pub fn map_unit_id_to_monster_id(&mut self, unit_id: u32, unit: &ESOLogsUnit) -> bool {
        return self.eso_logs_log.map_unit_id_to_monster_id(unit_id, unit)
    }

    pub fn add_object(&mut self, object: ESOLogsUnit) -> usize {
        return self.eso_logs_log.add_object(object)
    }

    pub fn add_buff(&mut self, buff: ESOLogsBuff) -> bool {
        return self.eso_logs_log.add_buff(buff)
    }

    pub fn add_buff_event(&mut self, buff_event: ESOLogsBuffEvent) -> usize {
        self.eso_logs_log.add_buff_event(buff_event)
    }

    pub fn unit_index(&self, unit_id: u32) -> Option<usize> {
        if unit_id == 0 {return Some(usize::MAX)}
        self.eso_logs_log.unit_index(&unit_id)
    }

    pub fn object_index(&self, object_id: String) -> Option<usize> {
        self.eso_logs_log.object_index(object_id)
    }

    pub fn buff_index(&self, buff_id: u32) -> Option<usize> {
        if buff_id == 0 {return Some(usize::MAX)}
        self.eso_logs_log.buff_index(buff_id)
    }

    pub fn add_log_event(&mut self, event: ESOLogsEvent) {
        let timestamp = event_timestamp(&event);
        self.eso_logs_log.add_log_event(event);

        if let Some(event_ts) = timestamp {
            let max_diff_ms: u64 = 20;
            let mut still_pending = Vec::with_capacity(self.pending_death_events.len());

            for (death_ts, mut death_event) in self.pending_death_events.drain(..) {
                if event_ts >= death_ts + max_diff_ms {
                    let mut forward_match: Option<(usize, u64)> = None;
                    let mut backward_match: Option<(usize, u64)> = None;

                    if let (Some(dc_id), Some(dt_id)) = (get_cast_id(&death_event), get_target_id(&death_event)) {
                        for (idx, e) in self.eso_logs_log.events.iter().enumerate().rev() {
                            if !is_damage_event(e) { continue; }
                            if let (Some(c_id), Some(t_id), Some(ts)) =
                                (get_cast_id(e), get_target_id(e), event_timestamp(e))
                            {
                                if c_id == dc_id && t_id == dt_id {
                                    if ts >= death_ts {
                                        let diff = ts - death_ts;
                                        if diff <= max_diff_ms {
                                            let better = forward_match.map_or(true, |(_, best_ts)| diff < best_ts - death_ts);
                                            if better {
                                                forward_match = Some((idx, ts));
                                            }
                                        }
                                    } else {
                                        let diff = death_ts - ts;
                                        if diff <= max_diff_ms {
                                            let better = backward_match.map_or(true, |(_, best_ts)| diff < death_ts - best_ts);
                                            if better {
                                                backward_match = Some((idx, ts));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some((match_idx, match_ts)) = forward_match.or(backward_match) {
                        set_event_timestamp(&mut death_event, match_ts);
                        self.eso_logs_log.events.insert(match_idx + 1, death_event);
                    } else {
                        let insert_idx = self.eso_logs_log.events.iter()
                            .position(|e| event_timestamp(e).map_or(false, |ts| ts > death_ts))
                            .unwrap_or(self.eso_logs_log.events.len());
                        self.eso_logs_log.events.insert(insert_idx, death_event);
                    }
                } else {
                    still_pending.push((death_ts, death_event));
                }
            }
            self.pending_death_events = still_pending;
        }
    }

    pub fn get_cp_for_unit(&self, unit_id: u32) -> u16 {
        self.eso_logs_log.get_cp_for_unit(unit_id)
    }

    pub fn index_in_session(&mut self, unit_id: u32) -> Option<usize> {
        self.eso_logs_log.index_in_session(&unit_id)
    }

    pub fn get_reaction_for_unit(&self, unit_id: u32) -> Option<Reaction> {
        self.eso_logs_log.get_reaction_for_unit(unit_id)
    }

    pub fn allegiance_from_reaction(reaction: Reaction) -> u8 {
        match reaction {
            Reaction::PlayerAlly | Reaction::NpcAlly => 16,
            Reaction::Hostile => 64,
            _ => 32,
        }
    }

    pub fn allegiance_from_unit_state(&self, unit_state: UnitState) -> Reaction {
        if unit_state == blank_unit_state() {return Reaction::None}
        let reaction = self.get_reaction_for_unit(unit_state.unit_id);
        return reaction.unwrap_or(Reaction::None);
    }

    pub fn convert_log_file_to_esolog_format(&mut self, file_path: &Path) -> Result<(), Box<dyn Error>> {
        let file: File = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = 0;
        let mut custom_log_data = CustomLogData::new();

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Error reading line: {}", e);
                    continue;
                }
            };
            let modified_line = handle_line(line, &mut custom_log_data);
            for line in modified_line {
                self.handle_line(line);
            }
            lines += 1;
            if lines % 250_000 == 0 {
                println!("Processed {} lines", lines);
                println!("Length of stuff: buffs:{}, effects:{}, units:{}, lines:{}", self.eso_logs_log.buffs.len(), self.eso_logs_log.effects.len(), self.eso_logs_log.units.len(), self.eso_logs_log.events.len());
            }
        }

        Ok(())
    }

    pub fn handle_line(&mut self, line: String) {
        let parts: Vec<String> = parser::parse::handle_line(&line);

        self.parse_line(&parts);
    }

    fn parse_line(&mut self, parts: &[String]) {
        let event = parts.get(1).map(|s| EventType::from(s.as_str())).unwrap_or(EventType::Unknown);

        match event {
            EventType::BeginLog      => self.handle_begin_log(parts),
            EventType::EndCombat     => self.handle_end_combat(parts),
            EventType::BeginCombat   => self.handle_begin_combat(parts),
            EventType::UnitAdded     => self.handle_unit_added(parts),
            EventType::PlayerInfo    => self.handle_player_info(parts),
            EventType::AbilityInfo   => self.handle_ability_info(parts),
            EventType::CombatEvent   => self.handle_combat_event(parts),
            EventType::BeginCast     => self.handle_begin_cast(parts),
            EventType::EffectChanged => self.handle_effect_changed(parts),
            EventType::MapChanged    => self.handle_map_changed(parts),
            EventType::ZoneChanged   => self.handle_zone_changed(parts),
            EventType::EndTrial      => self.handle_trial_end(parts),
            EventType::HealthRegen   => self.handle_health_recovery(parts),
            EventType::EffectInfo    => self.handle_effect_info(parts),
            EventType::UnitChanged   => self.handle_unit_changed(parts),
            EventType::EndCast       => self.handle_end_cast(parts),
            EventType::Unknown       => {}
        }
    }

    fn calculate_timestamp(&mut self, rel_ticks: u64) -> u64 {
        self.most_recent_begin_log_timestamp.unwrap_or(0) - self.base_timestamp.unwrap_or(0) + rel_ticks - self.timestamp_offset
    }

    fn buffs_pair_mut(&mut self, idx_a: usize, idx_b: usize) -> Option<(&mut ESOLogsBuff, &mut ESOLogsBuff)> {
        if idx_a == idx_b || idx_a >= self.eso_logs_log.buffs.len() || idx_b >= self.eso_logs_log.buffs.len() {
            return None;
        }

        let (small, large, flip) = if idx_a < idx_b {
            (idx_a, idx_b, false)
        } else {
            (idx_b, idx_a, true)
        };

        let (left, right) = self.eso_logs_log.buffs.split_at_mut(large);
        let (first, second) = (&mut left[small], &mut right[0]);

        Some(if flip { (second, first) } else { (first, second) })
    }

    fn handle_begin_log(&mut self, parts: &[String]) {
        self.megaserver = parts[4].to_owned();
        let log_ts = parts[2].parse::<u64>().unwrap();
        let rel_ticks = parts[0].parse::<u64>().unwrap();
        self.most_recent_begin_log_timestamp = Some(log_ts);
        self.timestamp_offset = rel_ticks;

        if self.base_timestamp.is_none() {
            self.base_timestamp = Some(log_ts);
        }
    }

    fn handle_end_combat(&mut self, parts: &[String]) {
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().unwrap());
        self.add_log_event(ESOLogsEvent::EndCombat(
            ESOLogsCombatEvent {
                timestamp,
                line_type: ESOLogsLineType::EndCombat,
            }
        ));
        self.eso_logs_log.session_units = HashMap::new();
        self.eso_logs_log.unit_index_in_session = HashMap::new();
    }

    fn handle_begin_combat(&mut self, parts: &[String]) {
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().unwrap());
        self.add_log_event(ESOLogsEvent::BeginCombat(
            ESOLogsCombatEvent {
                timestamp,
                line_type: ESOLogsLineType::BeginCombat,
            }
        ));
    }

    const BOSS_CLASS_ID: u8 = 100;
    const PET_CLASS_ID: u8 = 50;
    const OBJECT_CLASS_ID: u8 = 0;
    fn handle_unit_added(&mut self, parts: &[String]) {
        let event = parts.get(3).map(|s| UnitAddedEventType::from(s.as_str())).unwrap_or(UnitAddedEventType::Unknown);
        match event {
            UnitAddedEventType::Player => {
                let player = parse::player(parts);

                let unit = ESOLogsUnit {
                    name: player.name.trim_matches('"').to_owned(),
                    player_data: Some(ESOLogsPlayerSpecificData {
                        username: player.display_name,
                        character_id: player.character_id,
                        is_logging_player: player.is_local_player,
                    }),
                    unit_type: Reaction::PlayerAlly,
                    unit_id: player.player_per_session_id,
                    class: match player.class_id {
                            Class::None => 0,
                            Class::Dragonknight => 1,
                            Class::Sorcerer => 2,
                            Class::Nightblade => 3,
                            Class::Warden => 4,
                            Class::Necromancer => 5,
                            Class::Templar => 6,
                            Class::Arcanist => 117,
                    },
                    server_string: self.megaserver.clone(),
                    race: player.race_id,
                    icon: None,
                    champion_points: player.champion_points,
                    owner_id: 0,
                };
                self.map_unit_id_to_monster_id(player.unit_id, &unit);
                self.eso_logs_log.shield_values.insert(player.unit_id, 0);
                if let Some(unit_index) = self.eso_logs_log.session_id_to_units_index.get(&unit.unit_id) {
                    if let Some(player) = self.eso_logs_log.units.get_mut(*unit_index) {
                        if player.name == "Offline" {
                            *player = unit.clone();
                        }
                    }
                }
                self.eso_logs_log.players.insert(player.unit_id, true);
                self.eso_logs_log.players.insert(player.player_per_session_id, true);
                let index = self.add_unit(unit);
                self.eso_logs_log.unit_id_to_units_index.insert(player.unit_id, index);
            }
            UnitAddedEventType::Monster => {
                let monster = parse::monster(parts);
                // let name = monster.name.clone();
                // for unit in &self.eso_logs_log.units {
                //     if unit.name == name && unit.class == Self::BOSS_CLASS_ID {
                //         monster.is_boss = true;
                //         println!("Overwriting {} as boss", name);
                //     }
                // }
                let unit = ESOLogsUnit {
                    name: monster.name.trim_matches('"').to_owned(),
                    player_data: None,
                    unit_type: monster.reaction,
                    unit_id: monster.monster_id,
                    class: if monster.is_boss {Self::BOSS_CLASS_ID} else if monster.owner_unit_id != 0 {Self::PET_CLASS_ID} else {Self::OBJECT_CLASS_ID},
                    server_string: self.megaserver.clone(),
                    race: Race::None,
                    icon: None,
                    champion_points: monster.champion_points,
                    owner_id: monster.owner_unit_id,
                };
                let pet_owner_index = if monster.owner_unit_id != 0 {self.unit_index(monster.owner_unit_id)} else {None};
                self.map_unit_id_to_monster_id(monster.unit_id, &unit);
                if monster.is_boss {
                    self.eso_logs_log.bosses.insert(unit.unit_id, true);
                    self.eso_logs_log.bosses.insert(monster.unit_id, true);
                }
                let index = self.add_unit(unit);
                self.eso_logs_log.unit_id_to_units_index.insert(monster.unit_id, index);
                self.eso_logs_log.shield_values.insert(monster.unit_id, 0);
                if pet_owner_index.is_some() && monster.reaction == Reaction::NpcAlly {
                    let pet_relationship = ESOLogsPetRelationship {
                        owner_index: pet_owner_index.unwrap(),
                        pet: ESOLogsPet { pet_type_index: self.unit_index(monster.unit_id).unwrap_or(0) }
                    };
                    if !self.eso_logs_log.pets.iter().any(|rel| rel.owner_index == pet_relationship.owner_index && rel.pet.pet_type_index == pet_relationship.pet.pet_type_index) {
                        self.eso_logs_log.pets.push(pet_relationship);
                    }
                }
            }
            UnitAddedEventType::Object | UnitAddedEventType::SiegeWeapon => {
                let object = parse::object(parts);
                let unit = ESOLogsUnit {
                    name: object.name.trim_matches('"').to_owned(),
                    player_data: None,
                    unit_type: object.reaction,
                    unit_id: object.unit_id,
                    class: Self::OBJECT_CLASS_ID,
                    server_string: self.megaserver.clone(),
                    race: Race::None,
                    icon: None,
                    champion_points: object.champion_points,
                    owner_id: object.owner_unit_id,
                };
                self.map_unit_id_to_monster_id(object.unit_id, &unit);
                let index = self.add_object(unit);
                self.eso_logs_log.unit_id_to_units_index.insert(object.unit_id, index);
                self.eso_logs_log.shield_values.insert(object.unit_id, 0);
            }
            _ => ()
        }
    }

    fn handle_player_info(&mut self, parts: &[String]) {
        let length = parts.len();
        if length < 8 {
            eprintln!("Invalid PLAYER_INFO line: {:?}", parts);
            return;
        }

        // println!("Parts: {:?}", parts);
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().unwrap());
        
        self.add_log_event(ESOLogsEvent::PlayerInfo(
            ESOLogsPlayerBuild {
                timestamp,
                line_type: ESOLogsLineType::PlayerInfo,
                unit_index: self.unit_index(parts[2].parse().unwrap()).unwrap(),
                permanent_buffs: parts[3].to_string(),
                buff_stacks: parts[4].to_string(),
                gear: parts[5..length-2].iter().map(|s| s.to_string()).collect(),
                primary_abilities: parts[length-2].to_string(),
                backup_abilities: parts[length-1].to_string(),
            }
        ));
    }

    fn handle_ability_info(&mut self, parts: &[String]) {
        let ability = parse::ability(parts);
        let interruptible_blockable = (ability.interruptible as u8) * 2 + (ability.blockable as u8);
        let damage_type = match ability.id {
            _ => DamageType::None,
        };
        let buff = ESOLogsBuff {
            name: ability.name,
            damage_type,
            status_type: StatusEffectType::None,
            id: ability.id,
            icon: ability.icon.strip_suffix(".png").map(|s| s.to_string()).unwrap_or_else(|| ability.icon),
            caused_by_id: 0,
            interruptible_blockable: interruptible_blockable,
        };
        self.add_buff(buff);
    }

    fn update_shield_history(esolog: &mut ESOLogsLog, unit_id: u32, shield: u32, buff_event: &ESOLogsBuffEventKey2) {
        let units_stored_shield = *esolog.shield_values.get(&unit_id).unwrap_or(&0);
        let buff = &esolog.buffs[buff_event.buff_index];
        if shield != units_stored_shield || buff.id == 146311 /* frost safeguard */ { 
            // println!("Comparing shields for unit {}: {} new vs stored {}", unit_id, shield, units_stored_shield);
            if let Some(shield_buffs_for_unit) = esolog.shields.get_mut(&unit_id) {
                shield_buffs_for_unit.insert(buff_event.buff_index, buff_event.clone());
                // println!("Adding buff index: {}", buff_event.buff_index);
            } else {
                let mut hashmap = HashMap::new();
                hashmap.insert(buff_event.buff_index, buff_event.clone());
                esolog.shields.insert(unit_id, hashmap);
                // println!("Adding buff index: {}", buff_event.buff_index);
            }
            esolog.shield_values.insert(unit_id, shield);
        }
    }

    fn handle_combat_event(&mut self, parts: &[String]) {
        // println!("{:?}", parts);
        let source = parse::unit_state(parts, 9);
        let target = if parts[19] == "*" {
            source.clone()
        } else {
            parse::unit_state(parts, 19)
        };
        let ability_id = parts[8].parse().unwrap();
        if ability_id == 0 {return}
        let mut buff_event= ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index: self.unit_index(source.unit_id).expect("Combat event: source_unit_index should never be nothing"),
            target_unit_index: self.unit_index(target.unit_id).expect("Combat event: target_unit_index should never be nothing"),
            buff_index: self.buff_index(ability_id).expect("Combat event: buff_index should never be nothing"),
        };
        let unique_index = self.add_buff_event(buff_event);
        buff_event.unique_index = unique_index;
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().unwrap());
        let ev = event::Event {
            time: timestamp,
            result: event::parse_event_result(&parts[2]).unwrap(),
            damage_type: event::parse_damage_type(&parts[3]),
            power_type: parts[4].parse().unwrap(),
            hit_value: parts[5].parse().unwrap(),
            overflow: parts[6].parse().unwrap(),
            cast_track_id: parts[7].parse().unwrap(),
            ability_id: ability_id,
            source_unit_state: source,
            target_unit_state: target,
        };
        self.last_known_timestamp = ev.time;
        let critical = match ev.result {
                EventResult::Damage | EventResult::BlockedDamage | EventResult::DotTick | EventResult::Immune => 1,
                EventResult::CriticalDamage | EventResult::DotTickCritical => 2,
                EventResult::HotTick | EventResult::Heal => 1,
                EventResult::HotTickCritical | EventResult::CriticalHeal => 2,
                _ => 0,
            };
        let icon = self.eso_logs_log.get_buff_icon(ability_id);
        let cast_id = ev.cast_track_id;
        let index_option = self.eso_logs_log.buffs_hashmap.get(&cast_id).copied();
        let mut source_allegiance = Self::allegiance_from_reaction(self.allegiance_from_unit_state(source));
        let target_allegiance = Self::allegiance_from_reaction(self.allegiance_from_unit_state(target));

        match ev.result {
            EventResult::DamageShielded => {
                self.temporary_damage_buffer = ev.hit_value;
                let instance_ids = (
                    self.index_in_session(source.unit_id).unwrap_or(0),
                    self.index_in_session(target.unit_id).unwrap_or(0),
                );

                let (original_src_id, original_tgt_id) = self
                    .eso_logs_log
                    .shields
                    .get(&target.unit_id)
                    .and_then(|shields| shields.get(&buff_event.buff_index))
                    .map(|original| (original.source_unit_id, original.target_unit_id))
                    .unwrap_or((u32::MAX, u32::MAX));

                let shield_instance_ids = (
                    self.index_in_session(original_src_id).unwrap_or(0),
                    self.index_in_session(original_tgt_id).unwrap_or(0),
                );

                if cast_id != 0 {
                    if let Some(shield_buff_event) = self
                            .eso_logs_log
                            .shields
                            .get(&target.unit_id)
                            .and_then(|shields| shields.get(&buff_event.buff_index))
                            .cloned()
                        {
                            if let Some(original_shield_event) = self
                                .eso_logs_log
                                .shields
                                .get(&target.unit_id)
                                .and_then(|shields| shields.get(&buff_event.buff_index))
                            {
                                let new_key = ESOLogsBuffEventKey {
                                    source_unit_index: original_shield_event.source_unit_index,
                                    target_unit_index: original_shield_event.target_unit_index,
                                    buff_index: original_shield_event.buff_index,
                                };

                                if let Some(shield_event) = self.eso_logs_log.effects_hashmap.get(&new_key) {
                                    if let Some(buff_event_shield) = self.eso_logs_log.effects.get(*shield_event) {
                                        self.add_log_event(ESOLogsEvent::DamageShielded(ESOLogsDamageShielded {
                                            timestamp: ev.time,
                                            line_type: ESOLogsLineType::DamageShielded,
                                            buff_event: *buff_event_shield,
                                            shield_source_allegiance: Self::allegiance_from_reaction(
                                                self.eso_logs_log.units[buff_event_shield.source_unit_index].unit_type,
                                            ),
                                            shield_recipient_allegiance: target_allegiance,
                                            damage_source_allegiance: source_allegiance,
                                            unit_instance_id: instance_ids,
                                            orig_shield_instance_ids: shield_instance_ids,
                                            hit_value: ev.hit_value,
                                            source_ability_cast_index: index_option.unwrap_or(0),
                                        }));
                                    } else {
                                        println!("error: {:?}", shield_buff_event);
                                        println!("parts: {:?}", parts);
                                    }
                                }
                            }

                            Self::update_shield_history(
                                &mut self.eso_logs_log,
                                target.unit_id,
                                target.shield,
                                &shield_buff_event,
                            );
                        }
                    }
            }
            EventResult::BlockedDamage => {
                if let Some(buff) = self.eso_logs_log.buffs.get_mut(buff_event.buff_index) {
                    if ev.damage_type != DamageType::Heal { // You can't block a heal
                        buff.damage_type = ev.damage_type;
                    }
                }
                let instance_ids = (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0));
                self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: ev.time,
                        line_type: ESOLogsLineType::Damage,
                        buff_event: buff_event,
                        unit_instance_id: instance_ids,
                        cast: ESOLogsCastBase {
                            source_allegiance,
                            target_allegiance,
                            cast_id_origin: ev.cast_track_id,
                            source_unit_state: ESOLogsUnitState {
                                unit_state: source,
                                champion_points: self.get_cp_for_unit(source.unit_id),
                            },
                            target_unit_state: ESOLogsUnitState {
                                unit_state: target,
                                champion_points: self.get_cp_for_unit(target.unit_id),
                            },
                        },
                        cast_information: Some(ESOLogsCastData {
                            critical,
                            hit_value: ev.hit_value,
                            overflow: ev.overflow + self.temporary_damage_buffer,
                            override_magic_number: None,
                            replace_hitvalue_overflow: self.temporary_damage_buffer != 0,
                            blocked: true,
                        })
                    }
                ));
            }
            EventResult::Dodged => {
                if let Some(buff) = self.eso_logs_log.buffs.get_mut(buff_event.buff_index) {
                    if ev.damage_type != DamageType::Heal { // You can't dodge a heal
                        buff.damage_type = ev.damage_type;
                    }
                }
                let instance_ids = (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0));
                self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: ev.time,
                        line_type: ESOLogsLineType::Damage,
                        buff_event: buff_event,
                        unit_instance_id: instance_ids,
                        cast: ESOLogsCastBase {
                            source_allegiance,
                            target_allegiance,
                            cast_id_origin: ev.cast_track_id,
                            source_unit_state: ESOLogsUnitState {
                                unit_state: source,
                                champion_points: self.get_cp_for_unit(source.unit_id),
                            },
                            target_unit_state: ESOLogsUnitState {
                                unit_state: target,
                                champion_points: self.get_cp_for_unit(target.unit_id),
                            },
                        },
                        cast_information: Some(ESOLogsCastData {
                            critical,
                            hit_value: ev.hit_value,
                            overflow: ev.overflow + self.temporary_damage_buffer,
                            override_magic_number: Some(7),
                            replace_hitvalue_overflow: self.temporary_damage_buffer != 0,
                            blocked: false,
                        })
                    }
                ));
            }
            EventResult::Damage | EventResult::DotTick | EventResult::CriticalDamage | EventResult::DotTickCritical => {
                if !self.eso_logs_log.bosses.contains_key(&source.unit_id)  && buff_event.source_unit_index < self.eso_logs_log.units.len() {
                    let unit = &mut self.eso_logs_log.units[buff_event.source_unit_index];
                    if icon != "nil" && icon != "death_recap_melee_basic" {
                        unit.icon = Some(icon.clone());
                    } else if unit.icon.is_none() {
                        unit.icon = Some("death_recap_melee_basic".to_string());
                    }
                }
                if buff_event.target_unit_index < self.eso_logs_log.units.len() {
                    let unit = &mut self.eso_logs_log.units[buff_event.target_unit_index];
                    if unit.icon.is_none() {
                        unit.icon = Some("death_recap_melee_basic".to_string());
                    }
                }
                if buff_event.source_unit_index < self.eso_logs_log.units.len() {
                    let unit = &mut self.eso_logs_log.units[buff_event.source_unit_index];
                    if unit.unit_type == Reaction::Friendly {
                        unit.unit_type = Reaction::Hostile
                    }
                }
                if let Some(buff) = self.eso_logs_log.buffs.get_mut(buff_event.buff_index) {
                    buff.damage_type = ev.damage_type;
                }
                let instance_ids = (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0));
                self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: ev.time,
                        line_type: match ev.result {
                            EventResult::Damage | EventResult::CriticalDamage => ESOLogsLineType::Damage,
                            _ => ESOLogsLineType::DotTick,
                        },
                        buff_event: buff_event,
                        unit_instance_id: instance_ids,
                        cast: ESOLogsCastBase {
                            source_allegiance,
                            target_allegiance,
                            cast_id_origin: ev.cast_track_id,
                            source_unit_state: ESOLogsUnitState {
                                unit_state: source,
                                champion_points: self.get_cp_for_unit(source.unit_id),
                            },
                            target_unit_state: ESOLogsUnitState {
                                unit_state: target,
                                champion_points: self.get_cp_for_unit(target.unit_id),
                            },
                        },
                        cast_information: Some(ESOLogsCastData {
                            critical,
                            hit_value: ev.hit_value.saturating_sub(self.temporary_damage_buffer),
                            overflow: ev.overflow + self.temporary_damage_buffer,
                            override_magic_number: None,
                            replace_hitvalue_overflow: self.temporary_damage_buffer != 0,
                            blocked: false,
                        })
                    }
                ));
            }
            EventResult::HotTick | EventResult::CriticalHeal | EventResult::Heal | EventResult::HotTickCritical => {
                if buff_event.source_unit_index < self.eso_logs_log.units.len() {
                    let unit = &mut self.eso_logs_log.units[buff_event.source_unit_index];
                    if icon != "nil" && icon != "death_recap_melee_basic" {
                        unit.icon = Some(icon.clone());
                    } else if unit.icon.is_none() {
                        unit.icon = Some("death_recap_melee_basic".to_string());
                    }
                }
                if let Some(buff) = self.eso_logs_log.buffs.get_mut(buff_event.buff_index) {
                    buff.damage_type = ev.damage_type;
                }
                let instance_ids = (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0));
                self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: ev.time,
                        line_type: match ev.result {
                            EventResult::HotTick | EventResult::HotTickCritical => {ESOLogsLineType::HotTick}
                            _ => {ESOLogsLineType::Heal}
                        },
                        buff_event: buff_event,
                        unit_instance_id: instance_ids,
                        cast: ESOLogsCastBase {
                            source_allegiance,
                            target_allegiance,
                            cast_id_origin: ev.cast_track_id,
                            source_unit_state: ESOLogsUnitState {
                                unit_state: source,
                                champion_points: self.get_cp_for_unit(source.unit_id),
                            },
                            target_unit_state: ESOLogsUnitState {
                                unit_state: target,
                                champion_points: self.get_cp_for_unit(target.unit_id),
                            },
                        },
                        cast_information: Some(ESOLogsCastData {
                            critical,
                            hit_value: ev.hit_value,
                            overflow: ev.overflow,
                            override_magic_number: None,
                            replace_hitvalue_overflow: false,
                            blocked: false,
                        })
                    }
                ));
            }
            EventResult::PowerEnergize => {
                self.add_log_event(ESOLogsEvent::PowerEnergize(
                    ESOLogsPowerEnergize {
                        timestamp: ev.time,
                        line_type: ESOLogsLineType::PowerEnergize,
                        buff_event: buff_event,
                        cast: ESOLogsCastBase {
                            source_allegiance,
                            target_allegiance,
                            cast_id_origin: ev.cast_track_id,
                            source_unit_state: ESOLogsUnitState {
                                unit_state: source,
                                champion_points: self.get_cp_for_unit(source.unit_id),
                            },
                            target_unit_state: ESOLogsUnitState {
                                unit_state: target,
                                champion_points: self.get_cp_for_unit(target.unit_id),
                            },
                        },
                        hit_value: ev.hit_value,
                        overflow: ev.overflow,
                        resource_type: match ev.power_type {
                            1 => ESOLogsResourceType::Magicka,
                            4 => ESOLogsResourceType::Stamina,
                            8 => ESOLogsResourceType::Ultimate,
                            _ => {eprintln!("Unknown power type: {}", ev.power_type); ESOLogsResourceType::Health},
                        },
                    }
                ));
                return
            }
            EventResult::Died | EventResult::DiedXP | EventResult::KillingBlow => {
                let instance_ids = (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0));
                if source_allegiance == 32 {source_allegiance = 64}
                let timestamp = ev.time+1;
                let death_event = ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: timestamp.clone(),
                        line_type: ESOLogsLineType::Death,
                        buff_event: buff_event,
                        unit_instance_id: instance_ids,
                        cast: ESOLogsCastBase {
                            source_allegiance,
                            target_allegiance,
                            cast_id_origin: ev.cast_track_id,
                            source_unit_state: ESOLogsUnitState {
                                unit_state: source,
                                champion_points: self.get_cp_for_unit(source.unit_id),
                            },
                            target_unit_state: ESOLogsUnitState {
                                unit_state: target,
                                champion_points: self.get_cp_for_unit(target.unit_id),
                            },
                        },
                        cast_information: None,
                    }
                );
                self.pending_death_events.push((timestamp, death_event));
                return;
            }
            EventResult::Immune => {
                let instance_ids = (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0));
                self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: ev.time,
                        line_type: ESOLogsLineType::Damage,
                        buff_event: buff_event,
                        unit_instance_id: instance_ids,
                        cast: ESOLogsCastBase {
                            source_allegiance,
                            target_allegiance,
                            cast_id_origin: ev.cast_track_id,
                            source_unit_state: ESOLogsUnitState {
                                unit_state: source,
                                champion_points: self.get_cp_for_unit(source.unit_id),
                            },
                            target_unit_state: ESOLogsUnitState {
                                unit_state: target,
                                champion_points: self.get_cp_for_unit(target.unit_id),
                            },
                        },
                        cast_information: Some(ESOLogsCastData {
                            critical,
                            hit_value: ev.hit_value,
                            overflow: ev.overflow,
                            override_magic_number: Some(10),
                            replace_hitvalue_overflow: false,
                            blocked: false,
                        })
                    }
                ));
            }
            EventResult::Interrupt => {
                self.last_interrupt = Some(target.unit_id);
            }
            _ => {}
        };
        if ev.result != EventResult::DamageShielded {
            self.temporary_damage_buffer = 0;
        }
        if ev.result != EventResult::Interrupt {
            self.last_interrupt = None;
        }
        if let Some(caused_by_idx) = index_option {
            let target_idx = buff_event.buff_index;
            if let Some((target_buff, source_buff)) = self.buffs_pair_mut(target_idx, caused_by_idx) {
                target_buff.caused_by_id = source_buff.id;
            } else if caused_by_idx == target_idx {
                if let Some(buff) = self.eso_logs_log.buffs.get_mut(target_idx) {
                    buff.caused_by_id = buff.id;
                }
            }
        };
    }

    fn handle_begin_cast(&mut self, parts: &[String]) {
        let source = parse::unit_state(parts, 6);
        let target = if parts[16] == "*" {
            source.clone()
        } else {
            parse::unit_state(parts, 16)
        };
        let ability_id = parts[5].parse().unwrap();
        let mut buff_event= ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index: self.unit_index(source.unit_id).expect("Begin cast: unit_index should never be nothing"),
            target_unit_index: self.unit_index(target.unit_id).expect("Begin cast: unit_index should never be nothing"),
            buff_index: self.buff_index(ability_id).expect("Begin cast: buff_index should never be nothing"),
        };
        let cast_id: u32 = parts[4].parse().unwrap();
        self.eso_logs_log.buffs_hashmap.insert(cast_id, buff_event.buff_index);
        buff_event.unique_index = self.add_buff_event(buff_event);
        self.eso_logs_log.cast_id_hashmap.insert(cast_id, buff_event.unique_index);
        if cast_id != 0 {
            if let Some(buff) = self.eso_logs_log.buffs.get_mut(buff_event.buff_index) {
                buff.caused_by_id = ability_id;
            }
        }
        self.active_casts.insert(source.unit_id, buff_event.buff_index);
        let cast_time = parts[2].parse::<u32>().unwrap();
        if cast_time > 0 {
            self.eso_logs_log.cast_with_cast_time.insert(ability_id);
        }
        let source_allegiance = Self::allegiance_from_reaction(self.allegiance_from_unit_state(source));
        let target_allegiance = Self::allegiance_from_reaction(self.allegiance_from_unit_state(target));
        let instance_ids = (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0));
        self.eso_logs_log.cast_id_source_unit_id.insert(cast_id, source.unit_id);
        self.eso_logs_log.cast_id_target_unit_id.insert(cast_id, target.unit_id);
        if self.eso_logs_log.bosses.contains_key(&source.unit_id) {
            if source == target {
                if let Some(buff) = self.eso_logs_log.buffs.get(buff_event.buff_index) {
                    if let Some(unit_index) = self.eso_logs_log.unit_index(&source.unit_id) {
                        if let Some(unit) = self.eso_logs_log.units.get_mut(unit_index) {
                            unit.icon = Some(buff.icon.clone());
                        }
                    }
                }
            }
        }
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().unwrap());
        self.last_known_timestamp = timestamp;
        self.add_log_event(ESOLogsEvent::CastLine(
            ESOLogsCastLine {
                timestamp: self.last_known_timestamp,
                line_type: if cast_time > 0 {ESOLogsLineType::CastWithCastTime} else {ESOLogsLineType::Cast},
                buff_event: buff_event,
                unit_instance_id: instance_ids,
                cast: ESOLogsCastBase {
                    source_allegiance,
                    target_allegiance,
                    cast_id_origin: parts[4].parse().unwrap(),
                    source_unit_state: ESOLogsUnitState {
                        unit_state: source,
                        champion_points: self.get_cp_for_unit(source.unit_id),
                    },
                    target_unit_state: ESOLogsUnitState {
                        unit_state: target,
                        champion_points: self.get_cp_for_unit(target.unit_id),
                    },
                },
                cast_information: None,
            }
        ));
    }

    fn handle_effect_changed(&mut self, parts: &[String]) {
        // println!("{:?}", parts);
        let source = parse::unit_state(parts, 6);
        let target_equal_source = parts[16] == "*";
        let target = if target_equal_source {
            source.clone()
        } else {
            parse::unit_state(parts, 16)
        };
        let ability_id = parts[5].parse().unwrap();
        let mut buff_event= ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index: self.unit_index(source.unit_id).unwrap_or_else(|| panic!("Effect changed: source_unit_index should never be nothing. {:?}\n{:?}",parts, self.eso_logs_log.units)),
            target_unit_index: self.unit_index(target.unit_id).unwrap_or_else(|| panic!("Effect changed: target_unit_index should never be nothing. {:?}\n{:?}",parts, self.eso_logs_log.units)),
            buff_index: self.buff_index(ability_id).unwrap_or_else(|| panic!("Effect changed: buff_index should never be nothing. {:?}\n{:?}", parts, self.eso_logs_log.buffs)),
        };
        let cast_id: u32 = parts[4].parse().unwrap();
        buff_event.unique_index = self.add_buff_event(buff_event);
        let index = self.eso_logs_log.buffs_hashmap.get(&cast_id).copied();
        if let Some(caused_by_idx_raw) = index {
            let caused_by_idx = caused_by_idx_raw;
            let target_idx = buff_event.buff_index;
            if let Some((target_buff, source_buff)) = self.buffs_pair_mut(target_idx, caused_by_idx) {
                target_buff.caused_by_id = source_buff.id;
            } else if caused_by_idx == target_idx {
                if let Some(buff) = self.eso_logs_log.buffs.get_mut(target_idx) {
                    buff.caused_by_id = buff.id;
                }
            }
        };
        let mut source_allegiance = Self::allegiance_from_reaction(self.allegiance_from_unit_state(source));
        let mut target_allegiance = Self::allegiance_from_reaction(self.allegiance_from_unit_state(target));
        let shield_buff_event = ESOLogsBuffEventKey2 {
            source_unit_index: buff_event.source_unit_index,
            source_unit_id: source.unit_id,
            target_unit_index: buff_event.target_unit_index,
            target_unit_id: target.unit_id,
            buff_index: buff_event.buff_index,
        };
        if source.unit_id != 0 {
            Self::update_shield_history(&mut self.eso_logs_log, source.unit_id, source.shield, &shield_buff_event);
            Self::update_shield_history(&mut self.eso_logs_log, target.unit_id, target.shield, &shield_buff_event);
        }
        let instance_ids = (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0));
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().unwrap());
        self.last_known_timestamp = timestamp;
        if parts[2] == "GAINED" {
            self.add_log_event(ESOLogsEvent::BuffLine (
                ESOLogsBuffLine {
                    timestamp: self.last_known_timestamp,
                    line_type: if source_allegiance == target_allegiance {ESOLogsLineType::BuffGainedAlly} else {ESOLogsLineType::BuffGainedEnemy},
                    buff_event: buff_event,
                    unit_instance_id: instance_ids,
                    source_allegiance,
                    target_allegiance,
                    source_cast_index: index,
                    source_shield: source.shield,
                    target_shield: target.shield,
                }
            ));
        } else if parts[2] == "FADED" {
            self.add_log_event(ESOLogsEvent::BuffLine (
                ESOLogsBuffLine {
                    timestamp: self.last_known_timestamp,
                    line_type: if source_allegiance == target_allegiance {ESOLogsLineType::BuffFadedAlly} else {ESOLogsLineType::BuffFadedEnemy},
                    buff_event: buff_event,
                    unit_instance_id: instance_ids,
                    source_allegiance,
                    target_allegiance,
                    source_cast_index: None,
                    source_shield: source.shield,
                    target_shield: target.shield,
                }
            ));
        } else if parts[2] == "UPDATED" {
            let stacks = parts[3].parse::<u16>().unwrap_or(1);
            if stacks > 1 {
                if source_allegiance == 32 {source_allegiance = 16}
                if target_allegiance == 32 {target_allegiance = 16}
                self.add_log_event(ESOLogsEvent::StackUpdate (
                    ESOLogsBuffStacks {
                        timestamp: self.last_known_timestamp,
                        line_type: if source_allegiance == target_allegiance {ESOLogsLineType::BuffStacksUpdatedAlly} else {ESOLogsLineType::BuffStacksUpdatedEnemy},
                        buff_event: buff_event,
                        unit_instance_id: instance_ids,
                        source_allegiance,
                        target_allegiance,
                        stacks,
                    }
                ));
            }
        }
    }

    fn handle_map_changed(&mut self, parts: &[String]) {
        let zone_id = parts[2].parse().unwrap_or(0);
        let zone_name = parts[3].to_string().trim_matches('"').to_string();
        let map_url = parts[4].trim_matches('"').to_lowercase();
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().unwrap());
        self.add_log_event(ESOLogsEvent::MapInfo(ESOLogsMapInfo {
            timestamp,
            line_type: ESOLogsLineType::MapInfo,
            map_id: zone_id,
            map_name: zone_name,
            map_image_url: map_url,
        }));
    }

    fn handle_zone_changed(&mut self, parts: &[String]) {
        let zone_id: u16 = parts[2].parse().unwrap_or(0);
        let zone_name = parts[3].to_string().trim_matches('"').to_string();
        let difficulty: String = parts[4].trim_matches('"').to_string();
        let difficulty_int = match difficulty.as_str() {
            "NONE" => 0,
            "NORMAL" => 1,
            "VETERAN" => 2,
            _ => {
                eprintln!("Unknown zone difficulty: {}", difficulty);
                0
            }
        };
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().unwrap());
        self.add_log_event(ESOLogsEvent::ZoneInfo(ESOLogsZoneInfo {
            timestamp,
            line_type: ESOLogsLineType::ZoneInfo,
            zone_id,
            zone_name,
            zone_difficulty: difficulty_int,
        }));
    }

    fn handle_trial_end(&mut self, parts: &[String]) {
        let id = parts[3].parse::<u32>().unwrap_or(0);
        let duration = parts[4].parse::<u64>().unwrap_or(0);
        let success = parse::is_true(&parts[5]);
        let final_score = parts[6].parse::<u32>().unwrap_or(0);
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().unwrap());
        self.add_log_event(ESOLogsEvent::EndTrial(
            ESOLogsEndTrial {
                timestamp,
                line_type: ESOLogsLineType::EndTrial,
                trial_id: id as u8,
                duration,
                success: if success {1} else {0},
                final_score,
            }
        ));
    }

    const HEALTH_RECOVERY_BUFF_ID: u32 = 61322;
    fn handle_health_recovery(&mut self, parts: &[String]) {
        let health_recovery = ESOLogsBuff {
            name: "UseDatabaseName".to_string(),
            damage_type: DamageType::Heal,
            status_type: StatusEffectType::None,
            id: Self::HEALTH_RECOVERY_BUFF_ID,
            icon: "crafting_dom_beer_002".to_string(),
            caused_by_id: 0,
            interruptible_blockable: 0
        };
        self.add_buff(health_recovery);
        let source = parse::unit_state(parts, 3);
        let source_id = self.unit_index(source.unit_id).expect("health_recovery source_index should never be nothing");
        let buff_index = self.buff_index(Self::HEALTH_RECOVERY_BUFF_ID).expect("health_recovery_buff_index should always exist");
        let mut buff_event = ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index: source_id,
            target_unit_index: source_id,
            buff_index: buff_index,
        };
        let unique_index = self.add_buff_event(buff_event);
        buff_event.unique_index = unique_index;
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().unwrap());
        let health_recovery = ESOLogsHealthRecovery {
            timestamp,
            line_type: ESOLogsLineType::HotTick,
            buff_event: buff_event,
            effective_regen: parts[2].parse::<u32>().unwrap(),
            unit_state: ESOLogsUnitState { unit_state: source, champion_points: self.get_cp_for_unit(source.unit_id) }
        };
        self.add_log_event(ESOLogsEvent::HealthRecovery(health_recovery));
    }

    fn handle_effect_info(&mut self, parts: &[String]) {
        let effect_id: u32 = parts[2].parse().unwrap();
        // let effect_type = effect::parse_effect_type(parts[3]);
        let status_effect_type = effect::parse_status_effect_type(&parts[4]);
        if let Some(&idx) = self.eso_logs_log.buffs_hashmap.get(&effect_id) {
            if let Some(buff) = self.eso_logs_log.buffs.get_mut(idx) {
                buff.status_type = status_effect_type;
            }
        }
    }

    fn handle_unit_changed(&mut self, parts: &[String]) {
        let unit_id = parts[2].parse().unwrap();
        let unit_index = self.unit_index(unit_id);
        if unit_index.is_some() {
            let unit = &mut self.eso_logs_log.units[unit_index.unwrap()];
            unit.unit_type = unit::match_reaction(&parts[11]);
        }
    }

    fn handle_end_cast(&mut self, parts: &[String]) {
        let end_reason = parse_cast_end_reason(&parts[2]);
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().unwrap());
        if end_reason == Some(CastEndReason::Interrupted) {
            let interrupted_cast_id = parts[3].parse::<u32>().unwrap();
            let interrupted_ability = parts[4].parse::<u32>().unwrap();
            let interrupting_ability = parts[5].parse::<u32>().unwrap();
            let interrupting_unit = parts[6].parse::<u32>().unwrap(); // can be zero sometimes
            let mut target_id_option = self.eso_logs_log.cast_id_source_unit_id.get(&interrupted_cast_id).cloned();
                        
            if interrupting_unit == 0 {return}

            if target_id_option.is_none() {
                if let Some(last_interrupt) = &self.last_interrupt {
                    target_id_option = Some(last_interrupt.clone());
                } else {
                    println!("source for interrupted cast doesn't exist: {:?}", parts);
                    return;
                }
            }

            let target_id = target_id_option.unwrap().clone();
            let target_index = self.eso_logs_log.unit_index(&target_id).expect("every target id should map to a unit");
            let target = self.eso_logs_log.units.get(target_index).expect("every target index should be a unit");
            let target_allegiance = Self::allegiance_from_reaction(target.unit_type);
            let target_session_index = self.eso_logs_log.index_in_session(&target_id).unwrap_or(0);
            let source_index = self.unit_index(interrupting_unit).expect("interrupting unit should always exist");
            let source = self.eso_logs_log.units.get(source_index).expect("interrupting unit should always exist");
            let source_allegiance = Self::allegiance_from_reaction(source.unit_type);
            let instance_id = self.eso_logs_log.index_in_session(&interrupting_unit).unwrap_or(0);
            let interrupted_ability_index = self.buff_index(interrupted_ability).expect("interrupted ability should always be something");
            let interrupted_ability_from_table = self.eso_logs_log.buffs.get(interrupted_ability_index).expect("index should always be at a point into buffs");
            let interrupting_ability_index = self.buff_index(interrupting_ability).expect("interrupting ability should be something");
            let interrupting_ability_from_table = self.eso_logs_log.buffs.get(interrupting_ability_index).expect("index should always be at a point into buffs");

            if interrupting_ability_from_table.name == interrupted_ability_from_table.name {return}

            let new_cast_key = ESOLogsBuffEventKey {
                source_unit_index: source_index,
                target_unit_index: target_index,
                buff_index: interrupting_ability_index,
            };

            let buff_index = self.eso_logs_log.effects_hashmap.get(&new_cast_key);
            if let Some(index) = buff_index {
                let buff = self.eso_logs_log.effects.get(*index).expect("buff_index will always be valid index into buffs");

                self.add_log_event(ESOLogsEvent::Interrupt(
                ESOLogsInterrupt {
                    timestamp,
                    line_type: ESOLogsLineType::Interrupted,
                    buff_event: buff.clone(),
                    unit_instance_id: (instance_id, target_session_index),
                    source_allegiance,
                    target_allegiance,
                    interrupted_ability_index,
                }));
            }
        } else if end_reason == Some(CastEndReason::Completed) { // 249171,END_CAST,COMPLETED,6859448,37108
            let ability_cast_id = parts[3].parse::<u32>().unwrap();
            if !self.eso_logs_log.cast_with_cast_time.contains(&ability_cast_id) {return}
            // println!("Ability cast id: {}", ability_cast_id);
            // let completed_ability_id = parts[4].parse::<u32>().unwrap();
            let buff_index = self.eso_logs_log.cast_id_hashmap.get(&ability_cast_id).unwrap_or(&usize::MAX); // "buff from cast_id should always be something" except when it isn't
            if *buff_index == usize::MAX {return;}
            let buff = self.eso_logs_log.effects.get(*buff_index).expect("buff_index should always point to a buff event inside effects").clone();
            let caster_id_option = self.eso_logs_log.cast_id_source_unit_id.get(&ability_cast_id);
            
            if caster_id_option.is_none() {return}

            let caster_id = caster_id_option.unwrap().clone();
            let caster_index = self.eso_logs_log.unit_index(&caster_id)
                .expect("every target id should map to a unit");
            let caster = self.eso_logs_log.units
                .get(caster_index)
                .expect("every target index should be a unit");
            let caster_allegiance = Self::allegiance_from_reaction(caster.unit_type);
            let caster_session_index = self.eso_logs_log.index_in_session(&caster_id).unwrap_or(0);
            let target_id = self.eso_logs_log.cast_id_target_unit_id.get(&ability_cast_id).expect("every cast id should have a target").clone();
            let (target_allegiance, target_session_index) = if target_id != 0 {
                let target_index = self.eso_logs_log.unit_index(&target_id).expect("every target id should have an index");
                let target = self.eso_logs_log.units.get(target_index).expect("every target index should point to a unit");
                let target_allegiance = Self::allegiance_from_reaction(target.unit_type);
                let target_session_index = self.eso_logs_log.index_in_session(&target_id).unwrap_or(0);
                (target_allegiance, target_session_index)
            } else {
                let target_allegiance = caster_allegiance;
                let target_session_index = caster_session_index;
                (target_allegiance, target_session_index)
            };


            self.add_log_event(ESOLogsEvent::CastEnded(
                ESOLogsEndCast { 
                    timestamp,
                    line_type: ESOLogsLineType::Cast, 
                    buff_event: buff.clone(), 
                    unit_instance_id: (caster_session_index, target_session_index), 
                    source_allegiance: caster_allegiance, 
                    target_allegiance: target_allegiance 
                }
            ));
        }
    }
}

pub fn split_and_zip_log_by_fight<InputPath, OutputDir, F>(input_path: InputPath, output_dir: OutputDir, mut progress_callback: F) -> Result<(), String> where InputPath: AsRef<Path>, OutputDir: AsRef<Path>, F: FnMut(u8) {
    if output_dir.as_ref().exists() {
        fs::remove_dir_all(&output_dir)
            .map_err(|e| format!("Failed to remove existing output dir: {e}"))?;
    }
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output dir: {e}"))?;
    let timestamps_path = output_dir.as_ref().join("timestamps");
    if let Err(e) = fs::remove_file(&timestamps_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(format!("Failed to clear timestamps file: {e}"));
        }
    }

    let total_lines = {
        let file = File::open(&input_path)
            .map_err(|e| format!("Failed to open input file: {e}"))?;
        BufReader::new(file).lines().count()
    };

    let input_file = File::open(&input_path)
        .map_err(|e| format!("Failed to open input file: {e}"))?;
    let mut lines = BufReader::new(input_file).lines();

    let mut elp = ESOLogProcessor::new();
    let mut custom_state = CustomLogData::new();
    let mut fight_index: u16 = 1;

    let mut first_timestamp: Option<u64> = None;
    let mut current_line: usize = 0;
    while let Some(line) = lines.next() {
        current_line += 1;
        if current_line % LINE_COUNT_FOR_PROGRESS == 0 {
            progress_callback(((current_line as f64 / total_lines as f64) * 100.0).round() as u8);
        }

        let line = line.map_err(|e| format!("Read error: {e}"))?;
        let mut split = line.splitn(4, ',');
        let first = split.next();
        let second = split.next();
        let third = split.next();

        if let Some("BEGIN_LOG") = second {
            elp.eso_logs_log.new_log_reset();
            elp.reset();
            custom_state.reset();
            if let Some(third_str) = third {
                if let Ok(ts) = third_str.parse::<u64>() {
                    if let Some(first_str) = first {
                        if let Ok(ts2) = first_str.parse::<u64>() {
                            if first_timestamp.is_none() {first_timestamp = Some(ts + ts2)};
                        }
                    }
                }
            }
        }

        let is_end_combat = matches!(second, Some("END_COMBAT") | Some("END_LOG"));
        for l in handle_line(line, &mut custom_state) {
            elp.handle_line(l.to_string());
        }

        if is_end_combat {
            let seg_zip = output_dir
                .as_ref()
                .join(format!("report_segment_{fight_index}.zip"));
            let seg_data = build_report_segment(&elp);
            write_zip_with_logtxt(seg_zip, seg_data.as_bytes())?;

            let events = &elp.eso_logs_log.events;
            if !events.is_empty() {
                let mut last_ts = event_timestamp(&events[events.len()-1]);
                if last_ts.is_some() && first_timestamp.is_some() {
                    last_ts = Some(last_ts.unwrap() + first_timestamp.unwrap());
                }
                if let (Some(first), Some(last)) = (first_timestamp, last_ts) {
                    use std::io::Write;
                    let timestamps_path = output_dir.as_ref().join("timestamps");
                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(timestamps_path)
                        .map_err(|e| format!("Failed to open timestamps file: {e}"))?;
                    writeln!(file, "{},{}", first, last)
                        .map_err(|e| format!("Failed to write timestamps: {e}"))?;
                }
            }

            elp.eso_logs_log.events.clear();
            custom_state.reset();

            fight_index += 1;
        }
    }

    let tbl_zip = output_dir
        .as_ref()
        .join(format!("master_table_{fight_index}.zip"));
    let tbl_data = build_master_table(&mut elp);
    write_zip_with_logtxt(tbl_zip, tbl_data.as_bytes())?;

    Ok(())
}

pub fn write_zip_with_logtxt<P: AsRef<Path>>(zip_path: P, data: &[u8]) -> Result<(), String> {
    let file = File::create(&zip_path)
        .map_err(|e| format!("Failed to create `{}`: {e}", zip_path.as_ref().display()))?;
    let buf = BufWriter::new(file);
    let mut zip = ZipWriter::new(buf);

    zip.start_file(
        "log.txt",
        SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644),
    )
    .map_err(|e| format!("ZIP error (start_file): {e}"))?;

    zip.write_all(data)
        .map_err(|e| format!("Write error: {e}"))?;

    zip.finish()
        .map_err(|e| format!("ZIP error (finish): {e}"))?;
    Ok(())
}

pub fn build_report_segment(elp: &ESOLogProcessor) -> String {
    let mut out = String::with_capacity(elp.eso_logs_log.events.len() * 64);
    let server_id = if elp.megaserver == "NA Megaserver" { 1 } else { 2 };

    out.push_str(&format!("15|{}\n", server_id));
    out.push_str(&format!("{}\n", elp.eso_logs_log.events.len()));

    for e in &elp.eso_logs_log.events {
        out.push_str(&e.to_string());
        out.push('\n');
    }
    out
}

pub fn build_master_table(elp: &mut ESOLogProcessor) -> String {
    let approx_capacity = elp.eso_logs_log.units.len() * 128
        + elp.eso_logs_log.buffs.len() * 64
        + elp.eso_logs_log.effects.len() * 16;
    let mut out = String::with_capacity(approx_capacity);

    let default_icon = "ability_mage_065";
    let mut icon_by_name = std::collections::HashMap::<String, String>::new();
    for buff in elp.eso_logs_log.buffs.iter_mut() {
        let new_icon = match buff.id {
            135924 => Some("gear_seagiant_staff".to_string()), // RO cooldown
            193447 => Some("u38_antiquities_goldandblueshalknecklace".to_string()), // velothi
            189533 => Some("ability_arcanist_002".to_string()), // fatecarver
            188456 => Some("gear_undinfernium_head_a".to_string()), // ozezan
            154820 => Some("gear_rockgrove_heavy_head_a".to_string()), // saxhleel
            157738 => Some("gear_rockgrove_med_head_a".to_string()), // sul-xan
            111504 => Some("gear_undaunted_werewolfbehemoth_head_a".to_string()), // balorgh
            220015 => Some("gear_lucentguardian_heavy_head_a".to_string()), // lucent echoes
            147459 => Some("antiquities_ornate_necklace_3".to_string()), // pearls of ehlnofey
            _ => None,
        };
        if let Some(icon) = new_icon {
            buff.icon = icon;
        }
        if buff.icon != default_icon {
            icon_by_name.insert(buff.name.clone(), buff.icon.clone());
        }
    }
    for buff in elp.eso_logs_log.buffs.iter_mut() {
        if buff.icon == default_icon {
            if let Some(icon) = icon_by_name.get(&buff.name) {
                buff.icon = icon.clone();
                // buff.name = format!("{}*", buff.name);
            }
        }

        if matches!(buff.id,
            86304 // lifesteal
            | 172672 // whorl of the depths
            | 156020 // from the brink
            | 190960 // harmony (jewellery synergy)
            | 103966 // concentrated barrier
        ) {
            buff.caused_by_id = 0;
        }

        if buff.id == buff.caused_by_id && buff.damage_type == DamageType::None {
            buff.caused_by_id = 0;
        }
    }
    for i in 0..elp.eso_logs_log.buffs.len() {
        let child_damage_type = elp.eso_logs_log.buffs[i].damage_type;
        let parent_id = elp.eso_logs_log.buffs[i].caused_by_id;
        if parent_id == 0 || parent_id == elp.eso_logs_log.buffs[i].id {
            continue;
        }
        if let Some(&parent_idx) = elp.eso_logs_log.buffs_hashmap.get(&parent_id) {
            if let Some(parent_buff) = elp.eso_logs_log.buffs.get_mut(parent_idx) {
                if parent_buff.damage_type == DamageType::None {
                    parent_buff.damage_type = child_damage_type;
                } else if parent_buff.damage_type == DamageType::Heal
                    && child_damage_type != DamageType::Heal
                    && child_damage_type != DamageType::None
                {
                    parent_buff.damage_type = child_damage_type;
                }
            }
        }
    }

    let server_id = if elp.megaserver == "\"NA Megaserver\"" { 1 } else { 2 };
    out.push_str(&format!("15|{}|\n", server_id));

    out.push_str(&format!("{}\n", elp.eso_logs_log.units.len()));
    for u in &elp.eso_logs_log.units {
        out.push_str(&u.to_string());
        out.push('\n');
    }

    out.push_str(&format!("{}\n", elp.eso_logs_log.buffs.len()));
    for b in &elp.eso_logs_log.buffs {
        out.push_str(&b.to_string());
        out.push('\n');
    }

    out.push_str(&format!("{}\n", elp.eso_logs_log.effects.len()));
    for e in &elp.eso_logs_log.effects {
        out.push_str(&e.to_string());
        out.push('\n');
    }

    out.push_str(&format!("{}\n", elp.eso_logs_log.pets.len()));
    for p in &elp.eso_logs_log.pets {
        out.push_str(&p.to_string());
        out.push('\n');
    }
    out
}