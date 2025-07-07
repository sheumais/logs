use std::{collections::HashMap, error::Error, fs::File, io::{BufRead, BufReader, BufWriter}, path::Path};
use std::io::Write;
use parser::{effect::{self, StatusEffectType}, event::{self, DamageType, EventResult}, parse::{self}, player::{Class, Race}, unit::{self, blank_unit_state, Reaction, UnitState}};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};
use std::fs;

use crate::{esologs_format::*, log_edit::{handle_line, CustomLogData}};

pub struct ESOLogProcessor {
    pub eso_logs_log: ESOLogsLog,
    pub megaserver: String,
    pub timestamp_offset: u32,
}

impl ESOLogProcessor {
    pub fn new() -> Self {
        Self {
            eso_logs_log: ESOLogsLog::new(),
            megaserver: "Unknown".to_owned(),
            timestamp_offset: 0,
        }
    }

    pub fn add_unit(&mut self, unit: ESOLogsUnit) -> bool {
        return self.eso_logs_log.add_unit(unit)
    }

    pub fn map_unit_id_to_monster_id(&mut self, unit_id: u32, monster_id: u32) -> bool {
        return self.eso_logs_log.map_unit_id_to_monster_id(unit_id, monster_id)
    }

    pub fn add_object(&mut self, object: ESOLogsUnit) -> bool {
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
        self.eso_logs_log.unit_index(unit_id)
    }

    pub fn object_index(&self, object_id: String) -> Option<usize> {
        self.eso_logs_log.object_index(object_id)
    }

    pub fn buff_index(&self, buff_id: u32) -> Option<usize> {
        if buff_id == 0 {return Some(usize::MAX)}
        self.eso_logs_log.buff_index(buff_id)
    }

    pub fn add_log_event(&mut self, event: ESOLogsEvent) {
        self.eso_logs_log.add_log_event(event);
    }

    pub fn get_cp_for_unit(&self, unit_id: u32) -> u16 {
        self.eso_logs_log.get_cp_for_unit(unit_id)
    }

    pub fn index_in_session(&self, unit_id: u32) -> Option<usize> {
        self.eso_logs_log.index_in_session(unit_id)
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

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Error reading line: {}", e);
                    continue;
                }
            };
            self.handle_line(line);
            lines += 1;
            if lines % 250_000 == 0 {
                println!("Processed {} lines", lines);
                println!("Length of stuff: buffs:{}, effects:{}, units:{}, lines:{}", self.eso_logs_log.buffs.len(), self.eso_logs_log.effects.len(), self.eso_logs_log.units.len(), self.eso_logs_log.events.len());
            }
        }

        Ok(())
    }

    pub fn handle_line(&mut self, line: String) {
        let mut parts: Vec<&str> = Vec::new();
        let mut start = 0usize;
        let mut bracket_level = 0u32;
        let mut in_quotes = false;
        let bytes = line.as_bytes();

        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'"' => {
                    if i == 0 || bytes[i - 1] != b'\\' {
                        in_quotes = !in_quotes;
                    }
                }

                b'[' if !in_quotes => {
                    bracket_level += 1;
                    if i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                        i += 1;
                    }
                }

                b']' if !in_quotes => {
                    if bracket_level > 0 {
                        bracket_level -= 1;
                    }
                    if i + 1 < bytes.len() && bytes[i + 1] == b']' {
                        i += 1;
                    }
                }

                b',' if !in_quotes && bracket_level == 0 => {
                    let field = line[start..i]
                        .trim_matches(&['[', ']'][..])
                        .trim();
                    if !field.is_empty() {
                        parts.push(field);
                    }
                    start = i + 1;
                }
                _ => {}
            }
            i += 1;
        }

        if start < line.len() {
            let field = line[start..]
                .trim_matches(&['[', ']'][..])
                .trim();
            if !field.is_empty() {
                parts.push(field);
            }
        }

        self.parse_line(parts);
    }

    fn parse_line(&mut self, parts: Vec<&str>) {
        match parts.get(1) {
            Some(&"BEGIN_LOG") => self.handle_begin_log(parts),
            Some(&"END_COMBAT") => self.handle_end_combat(parts),
            Some(&"BEGIN_COMBAT") => self.handle_begin_combat(parts),
            Some(&"UNIT_ADDED") => self.handle_unit_added(&parts),
            Some(&"PLAYER_INFO") => self.handle_player_info(&parts),
            Some(&"ABILITY_INFO") => self.handle_ability_info(&parts),
            Some(&"COMBAT_EVENT") => self.handle_combat_event(&parts),
            Some(&"BEGIN_CAST") => self.handle_begin_cast(&parts),
            Some(&"EFFECT_CHANGED") => self.handle_effect_changed(&parts),
            Some(&"MAP_CHANGED") => self.handle_map_changed(&parts),
            Some(&"ZONE_CHANGED") => self.handle_zone_changed(&parts),
            Some(&"END_TRIAL") => self.handle_trial_end(&parts),
            Some(&"HEALTH_REGEN") => self.handle_health_recovery(&parts),
            Some(&"EFFECT_INFO") => self.handle_effect_info(&parts),
            Some(&"UNIT_CHANGED") => self.handle_unit_changed(&parts),
            _ => {},
        }
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

    fn handle_begin_log(&mut self, parts: Vec<&str>) {
        self.megaserver = parts[4].to_owned();
        self.eso_logs_log = ESOLogsLog::new();
    }

    fn handle_end_combat(&mut self, parts: Vec<&str>) {
        self.add_log_event(ESOLogsEvent::EndCombat(
            ESOLogsCombatEvent {
                timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
                line_type: ESOLogsLineType::EndCombat,
            }
        ));
    }

    fn handle_begin_combat(&mut self, parts: Vec<&str>) {
        self.add_log_event(ESOLogsEvent::BeginCombat(
            ESOLogsCombatEvent {
                timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
                line_type: ESOLogsLineType::BeginCombat,
            }
        ));
    }

    const BOSS_CLASS_ID: u8 = 100;
    const PET_CLASS_ID: u8 = 50;
    const OBJECT_CLASS_ID: u8 = 0;
    fn handle_unit_added(&mut self, parts: &[&str]) {
        match parts[3] {
            "PLAYER" => {
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
                    champion_points: player.champion_points
                };
                self.map_unit_id_to_monster_id(player.unit_id, player.player_per_session_id);
                self.eso_logs_log.shield_values.insert(player.unit_id, 0);
                self.eso_logs_log.players.insert(player.unit_id, true);
                if let Some(unit_index) = self.eso_logs_log.session_id_to_units_index.get(&unit.unit_id) {
                    if let Some(player) = self.eso_logs_log.units.get_mut(*unit_index) {
                        if player.name == "Offline" {
                            *player = unit.clone();
                        }
                    }
                }
                self.add_unit(unit);
            }
            "MONSTER" => {
                let monster = parse::monster(parts);
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
                };
                let pet_owner_index = self.unit_index(monster.owner_unit_id);
                self.add_unit(unit);
                self.map_unit_id_to_monster_id(monster.unit_id, monster.monster_id);
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
            "OBJECT" => {
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
                };
                self.add_object(unit);
                self.map_unit_id_to_monster_id(object.unit_id, object.unit_id);
                self.eso_logs_log.shield_values.insert(object.unit_id, 0);
            }
            &_ => ()
        }
    }

    fn handle_player_info(&mut self, parts: &[&str]) {
        let length = parts.len();
        if length < 8 {
            eprintln!("Invalid PLAYER_INFO line: {:?}", parts);
            return;
        }
        self.add_log_event(ESOLogsEvent::PlayerInfo(
            ESOLogsPlayerBuild {
                timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
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

    fn handle_ability_info(&mut self, parts: &[&str]) {
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
        let buff = &esolog.buffs[buff_event.buff_index as usize];
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

    fn handle_combat_event(&mut self, parts: &[&str]) {
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
        let ev = event::Event {
            time: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
            result: event::parse_event_result(parts[2]).unwrap(),
            damage_type: event::parse_damage_type(parts[3]),
            power_type: parts[4].parse().unwrap(),
            hit_value: parts[5].parse().unwrap(),
            overflow: parts[6].parse().unwrap(),
            cast_track_id: parts[7].parse().unwrap(),
            ability_id: ability_id,
            source_unit_state: source,
            target_unit_state: target,
        };
        let critical = match ev.result {
                EventResult::Damage | EventResult::BlockedDamage | EventResult::DotTick | EventResult::Immune => 1,
                EventResult::CriticalDamage | EventResult::DotTickCritical => 2,
                EventResult::HotTick | EventResult::Heal => 1,
                EventResult::HotTickCritical | EventResult::CriticalHeal => 2,
                _ => 0,
            };
        let icon = self.eso_logs_log.get_buff_icon(ability_id);
        let cast_id: u32 = ev.cast_track_id;
        let index_option = self.eso_logs_log.buffs_hashmap.get(&cast_id).copied();
        let source_allegiance = Self::allegiance_from_reaction(self.allegiance_from_unit_state(source));
        let target_allegiance = Self::allegiance_from_reaction(self.allegiance_from_unit_state(target));
        match ev.result {
            EventResult::DamageShielded => {
                if let Some(buff) = self.eso_logs_log.buffs.get_mut(buff_event.buff_index as usize) {
                    if ev.damage_type != DamageType::Heal {
                        buff.damage_type = ev.damage_type;
                    }
                }
                // if index_option.is_none() {
                //     println!("index is none - cast id {}", cast_id);
                //     if cast_id == 0 {
                //         println!("{:?}", parts);
                //     }
                // }
                if cast_id != 0 {
                    // println!("Looking for unit {} and ability {}", &target.unit_id, &ability_id);
                    let shield_buff_event = &self.eso_logs_log.shields.get(&target.unit_id).unwrap().get(&buff_event.buff_index).unwrap().clone();
                    if let Some(shields) = self.eso_logs_log.shields.get(&target.unit_id) {
                        if let Some(original_shield_event) = shields.get(&buff_event.buff_index) {
                            let new_key = ESOLogsBuffEventKey {
                                source_unit_index: original_shield_event.source_unit_index,
                                target_unit_index: original_shield_event.target_unit_index,
                                buff_index: original_shield_event.buff_index
                            };
                            let shield_event = self.eso_logs_log.effects_hashmap.get(&new_key).unwrap();
                            let option = self.eso_logs_log.effects.get(*shield_event);
                            if option.is_none() {
                                println!("error: {:?}", shield_buff_event);
                                println!("parts: {:?}", parts);
                            }
                            let buff_event_shield = option.unwrap();
                            // println!("event_shield = {:?}", buff_event_shield);
                            // println!("shield source = {:?}", self.eso_logs_log.units[buff_event_shield.source_unit_index as usize]);
                            self.add_log_event(ESOLogsEvent::DamageShielded(ESOLogsDamageShielded {
                                timestamp: ev.time, 
                                line_type: ESOLogsLineType::DamageShielded,
                                buff_event: *buff_event_shield, // buff event from shield history, unique to each ability id,
                                shield_source_allegiance: Self::allegiance_from_reaction(self.eso_logs_log.units[buff_event_shield.source_unit_index as usize].unit_type), // allegiance of buff event source,
                                shield_recipient_allegiance: target_allegiance,
                                damage_source_allegiance: source_allegiance,
                                unit_instance_id: (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0)),
                                orig_shield_instance_ids: (self.index_in_session(original_shield_event.source_unit_id).unwrap_or(0), self.index_in_session(original_shield_event.target_unit_id).unwrap_or(0)), // index in session of buff event source and target,
                                hit_value: ev.hit_value,
                                source_ability_cast_index: index_option.unwrap_or(self.eso_logs_log.buffs.len().wrapping_add(1)) // ability from ev.cast_track_id 
                            }));
                        }
                    }
                    Self::update_shield_history(&mut self.eso_logs_log, target.unit_id, target.shield, &shield_buff_event);
                }
            }
            EventResult::BlockedDamage | EventResult::Dodged => {
                if let Some(buff) = self.eso_logs_log.buffs.get_mut(buff_event.buff_index as usize) {
                    if ev.damage_type != DamageType::Heal { // You can't block or dodge a heal
                        buff.damage_type = ev.damage_type;
                    }
                }
            }
            EventResult::Damage | EventResult::DotTick | EventResult::CriticalDamage | EventResult::DotTickCritical => {
                if buff_event.source_unit_index < self.eso_logs_log.units.len() {
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
                if let Some(buff) = self.eso_logs_log.buffs.get_mut(buff_event.buff_index as usize) {
                    buff.damage_type = ev.damage_type;
                }
                self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: ev.time,
                        line_type: match ev.result {
                            EventResult::Damage | EventResult::CriticalDamage => ESOLogsLineType::Damage,
                            _ => ESOLogsLineType::DotTick,
                        },
                        buff_event: buff_event,
                        unit_instance_id: {
                            let src_idx = self.index_in_session(source.unit_id).unwrap_or(0);
                            let tgt_idx = self.index_in_session(target.unit_id).unwrap_or(0);
                            (src_idx, tgt_idx)
                        },
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
                if let Some(buff) = self.eso_logs_log.buffs.get_mut(buff_event.buff_index as usize) {
                    buff.damage_type = ev.damage_type;
                }
                self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: ev.time,
                        line_type: match ev.result {
                            EventResult::HotTick | EventResult::HotTickCritical => {ESOLogsLineType::HotTick}
                            _ => {ESOLogsLineType::Heal}
                        },
                        buff_event: buff_event,
                        unit_instance_id: (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0)),
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
                        resource_type: match ev.power_type { // health and stamina are intentionally around the wrong way from format
                            0 => ESOLogsResourceType::Magicka,
                            4 => ESOLogsResourceType::Stamina,
                            8 => ESOLogsResourceType::Ultimate,
                            1 => ESOLogsResourceType::Health,
                            _ => {eprintln!("Unknown power type: {}", ev.power_type); ESOLogsResourceType::Health},
                        },
                    }
                ));
                return
            }
            EventResult::Died | EventResult::DiedXP | EventResult::KillingBlow => {
                self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: ev.time,
                        line_type: ESOLogsLineType::Death,
                        buff_event: buff_event,
                        unit_instance_id: (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0)),
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
                            hit_value: if ev.hit_value == 0 {ev.overflow} else {ev.hit_value},
                            overflow: ev.overflow,
                        })
                    }
                ));
                return;
            }
            _ => {}
        };

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

    fn handle_begin_cast(&mut self, parts: &[&str]) {
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
        if cast_id != 0 {
            if let Some(buff) = self.eso_logs_log.buffs.get_mut(buff_event.buff_index as usize) {
                buff.caused_by_id = ability_id;
            }
        }
        let cast_time = parts[2].parse::<u32>().unwrap();
        let source_allegiance = Self::allegiance_from_reaction(self.allegiance_from_unit_state(source));
        let target_allegiance = Self::allegiance_from_reaction(self.allegiance_from_unit_state(target));
        self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
                        line_type: if cast_time > 0 {ESOLogsLineType::CastWithCastTime} else {ESOLogsLineType::Cast},
                        buff_event: buff_event,
                        unit_instance_id: (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0)),
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

    fn handle_effect_changed(&mut self, parts: &[&str]) {
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
            source_unit_index: self.unit_index(source.unit_id).expect("Effect changed: source_unit_index should never be nothing"),
            target_unit_index: self.unit_index(target.unit_id).expect("Effect changed: target_unit_index should never be nothing"),
            buff_index: self.buff_index(ability_id).expect("Effect changed: buff_index should never be nothing"),
        };
        let cast_id: u32 = parts[4].parse().unwrap();
        buff_event.unique_index = self.add_buff_event(buff_event);
        let index = self.eso_logs_log.buffs_hashmap.get(&cast_id).copied();
        if let Some(caused_by_idx_raw) = index {
            let caused_by_idx = caused_by_idx_raw as usize;
            let target_idx = buff_event.buff_index as usize;
            if let Some((target_buff, source_buff)) = self.buffs_pair_mut(target_idx, caused_by_idx) {
                target_buff.caused_by_id = source_buff.id;
            } else if caused_by_idx == target_idx {
                if let Some(buff) = self.eso_logs_log.buffs.get_mut(target_idx) {
                    buff.caused_by_id = buff.id;
                }
            }
        };
        let source_allegiance = Self::allegiance_from_reaction(self.allegiance_from_unit_state(source));
        let target_allegiance = Self::allegiance_from_reaction(self.allegiance_from_unit_state(target));
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

        if parts[2] == "GAINED" {
            self.add_log_event(ESOLogsEvent::BuffLine (
                ESOLogsBuffLine {
                    timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
                    line_type: if source_allegiance == target_allegiance {ESOLogsLineType::BuffGainedAlly} else {ESOLogsLineType::BuffGainedEnemy},
                    buff_event: buff_event,
                    unit_instance_id: (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0)),
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
                    timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
                    line_type: if source_allegiance == target_allegiance {ESOLogsLineType::BuffFadedAlly} else {ESOLogsLineType::BuffFadedEnemy},
                    buff_event: buff_event,
                    unit_instance_id: (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0)),
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
                self.add_log_event(ESOLogsEvent::StackUpdate (
                    ESOLogsBuffStacks {
                        timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
                        line_type: if source_allegiance == target_allegiance || (source_allegiance != 64 && target_allegiance != 64) {ESOLogsLineType::BuffStacksUpdatedAlly} else {ESOLogsLineType::BuffStacksUpdatedEnemy},
                        buff_event: buff_event,
                        unit_instance_id: (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0)),
                        source_allegiance,
                        target_allegiance,
                        stacks,
                    }
                ));
            }
        }
    }

    fn handle_map_changed(&mut self, parts: &[&str]) {
        let zone_id = parts[2].parse().unwrap_or(0);
        let zone_name = parts[3].to_string().trim_matches('"').to_string();
        let map_url = parts[4].trim_matches('"').to_lowercase();
        self.add_log_event(ESOLogsEvent::MapInfo(ESOLogsMapInfo {
            timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
            line_type: ESOLogsLineType::MapInfo,
            map_id: zone_id,
            map_name: zone_name,
            map_image_url: map_url,
        }));
    }

    fn handle_zone_changed(&mut self, parts: &[&str]) {
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
        self.timestamp_offset = parts[0].parse::<u32>().unwrap_or(0);
        self.add_log_event(ESOLogsEvent::ZoneInfo(ESOLogsZoneInfo {
            timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
            line_type: ESOLogsLineType::ZoneInfo,
            zone_id,
            zone_name,
            zone_difficulty: difficulty_int,
        }));
    }

    fn handle_trial_end(&mut self, parts: &[&str]) {
        let id = parts[3].parse::<u32>().unwrap_or(0);
        let duration = parts[4].parse::<u64>().unwrap_or(0);
        let success = parse::is_true(parts[5]);
        let final_score = parts[6].parse::<u32>().unwrap_or(0);
        self.add_log_event(ESOLogsEvent::EndTrial(
            ESOLogsEndTrial {
                timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
                line_type: ESOLogsLineType::EndTrial,
                trial_id: id as u8,
                duration,
                success: if success {1} else {0},
                final_score,
            }
        ));
    }

    const HEALTH_RECOVERY_BUFF_ID: u32 = 61322;
    fn handle_health_recovery(&mut self, parts: &[&str]) {
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
        let health_recovery = ESOLogsHealthRecovery {
            timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
            line_type: ESOLogsLineType::HotTick,
            buff_event: buff_event,
            effective_regen: parts[2].parse::<u32>().unwrap(),
            unit_state: ESOLogsUnitState { unit_state: source, champion_points: self.get_cp_for_unit(source.unit_id) }
        };
        self.add_log_event(ESOLogsEvent::HealthRecovery(health_recovery));
    }

    fn handle_effect_info(&mut self, parts: &[&str]) {
        let effect_id: u32 = parts[2].parse().unwrap();
        // let effect_type = effect::parse_effect_type(parts[3]);
        let status_effect_type = effect::parse_status_effect_type(parts[4]);
        if let Some(&idx) = self.eso_logs_log.buffs_hashmap.get(&effect_id) {
            if let Some(buff) = self.eso_logs_log.buffs.get_mut(idx) {
                buff.status_type = status_effect_type;
            }
        }
    }

    fn handle_unit_changed(&mut self, parts: &[&str]) {
        let unit_id = parts[2].parse().unwrap();
        let unit_index = self.unit_index(unit_id);
        if unit_index.is_some() {
            let unit = &mut self.eso_logs_log.units[unit_index.unwrap()];
            unit.unit_type = unit::match_reaction(parts[11]);
        }
    }
}

pub fn split_and_zip_log_by_fight<InputPath, OutputDir>(input_path: InputPath, output_dir: OutputDir) -> Result<(), String> where InputPath: AsRef<Path>, OutputDir: AsRef<Path> {
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output dir: {e}"))?;
    let timestamps_path = output_dir.as_ref().join("timestamps");
    if let Err(e) = fs::remove_file(&timestamps_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(format!("Failed to clear timestamps file: {e}"));
        }
    }
    let input_file = File::open(&input_path)
        .map_err(|e| format!("Failed to open input file: {e}"))?;
    let mut lines = BufReader::new(input_file).lines();

    let mut elp = ESOLogProcessor::new();
    let mut custom_state = CustomLogData::new();
    let mut fight_index: u16 = 1;

    let mut first_timestamp: Option<u64> = None;
    while let Some(line) = lines.next() {
        let line = line.map_err(|e| format!("Read error: {e}"))?;
        let mut split = line.splitn(4, ',');        
        let _first = split.next();
        let second = split.next();        
        let third = split.next();

        if let Some("BEGIN_LOG") = second {
            if let Some(third_str) = third {
                if let Ok(ts) = third_str.parse::<u64>() {
                    first_timestamp = Some(ts);
                    println!("setting first_timestamp to {}", first_timestamp.unwrap());
                    elp = ESOLogProcessor::new();
                }
            }
        }

        let is_end_combat = matches!(second, Some("END_COMBAT"));
        for l in handle_line(line, &mut custom_state) {
            elp.handle_line(l.to_string());
        }

        if is_end_combat {
            let seg_zip = output_dir
                .as_ref()
                .join(format!("report_segment_{fight_index}.zip"));
            let seg_data = build_report_segment(&elp);
            write_zip_with_logtxt(seg_zip, seg_data.as_bytes())?;

            let tbl_zip = output_dir
                .as_ref()
                .join(format!("master_table_{fight_index}.zip"));
            let tbl_data = build_master_table(&mut elp);
            write_zip_with_logtxt(tbl_zip, tbl_data.as_bytes())?;

            let events = &elp.eso_logs_log.events;
            if !events.is_empty() {
                fn get_timestamp(event: &ESOLogsEvent) -> Option<u64> {
                    match event {
                        ESOLogsEvent::BuffLine(e) => Some(e.timestamp),
                        ESOLogsEvent::CastLine(e) => Some(e.timestamp),
                        ESOLogsEvent::PowerEnergize(e) => Some(e.timestamp),
                        ESOLogsEvent::ZoneInfo(e) => Some(e.timestamp),
                        ESOLogsEvent::PlayerInfo(e) => Some(e.timestamp),
                        ESOLogsEvent::MapInfo(e) => Some(e.timestamp),
                        ESOLogsEvent::EndCombat(e) => Some(e.timestamp),
                        ESOLogsEvent::BeginCombat(e) => Some(e.timestamp),
                        ESOLogsEvent::EndTrial(e) => Some(e.timestamp),
                        _ => None,
                    }
                }
                let mut last_ts = get_timestamp(&events[events.len()-1]);
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

            fight_index += 1;
        }
    }

    Ok(())
}



fn write_zip_with_logtxt<P: AsRef<Path>>(zip_path: P, data: &[u8]) -> Result<(), String> {
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
    for buff in elp.eso_logs_log.buffs.iter() {
        if buff.icon != default_icon {
            icon_by_name.insert(buff.name.clone(), buff.icon.clone());
        }
    }
    for buff in elp.eso_logs_log.buffs.iter_mut() {
        if buff.icon == default_icon {
            if let Some(icon) = icon_by_name.get(&buff.name) {
                buff.icon = icon.clone();
            }
        }
        // lifesteal, whorl of the depths
        if matches!(buff.id, 86304 | 172672) {
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