use std::{error::Error, fs::File, io::{BufRead, BufReader}, path::Path};

use parser::{event::{self, DamageType, EventResult}, parse::{self}, player::{Class, Race}, unit::Reaction};

use crate::esologs_format::*;

pub struct ESOLogProcessor {
    pub eso_logs_log: ESOLogsLog,
    megaserver: String,
}

impl ESOLogProcessor {
    pub fn new() -> Self {
        Self {
            eso_logs_log: ESOLogsLog::new(),
            megaserver: "Unknown".to_owned(),
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
        self.eso_logs_log.unit_index(unit_id)
    }

    pub fn buff_index(&self, buff_id: u32) -> Option<usize> {
        self.eso_logs_log.buff_index(buff_id)
    }

    pub fn add_log_event(&mut self, event: ESOLogsEvent) {
        self.eso_logs_log.add_log_event(event);
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

    fn handle_line(&mut self, line: String) {
        let mut in_brackets = false;
        let mut in_quotes = false;
        let mut start = 0;
        let mut just_closed_quote = false; 
        let mut parts: Vec<&str> = Vec::new();

        let mut iter = line.char_indices().peekable();
        while let Some((i, ch)) = iter.next() {
            match ch {
                '[' if !in_quotes => { in_brackets = true;  start = i + 1; }
                ']' if !in_quotes => {
                    in_brackets = false;
                    parts.push(&line[start..i]);
                    start = i + 1;
                }

                '"' => {
                    if in_quotes && iter.peek().map(|(_,c)| *c) == Some('"') {
                        iter.next();
                        continue;
                    }

                    if in_quotes {
                        parts.push(&line[start..i]);
                        in_quotes = false;
                        just_closed_quote = true;
                        start = i + 1;
                    } else {
                        in_quotes = true;
                        start = i + 1;
                    }
                }

                ',' if !in_brackets && !in_quotes => {
                    if just_closed_quote {
                        just_closed_quote = false;
                        start = i + 1;
                    } else {
                        parts.push(&line[start..i]);
                        start = i + 1;
                    }
                }

                _ => {}
            }
        }

        if start < line.len() || just_closed_quote {
            parts.push(&line[start..]);
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
            Some(&"TRIAL_INIT") => self.handle_trial_init(&parts),
            Some(&"END_TRIAL") => self.handle_trial_end(&parts),
            Some(&"HEALTH_REGEN") => self.handle_health_recovery(&parts),
            _ => {},
        }
    }

    fn handle_begin_log(&mut self, parts: Vec<&str>) {
        self.megaserver = parts[4].to_owned();
    }

    fn handle_end_combat(&mut self, parts: Vec<&str>) {

    }

    fn handle_begin_combat(&mut self, parts: Vec<&str>) {

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
                if unit.name != "Offline" {
                    self.add_unit(unit);
                }
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
                self.map_unit_id_to_monster_id(monster.unit_id, monster.monster_id);
                self.add_unit(unit);
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
                self.map_unit_id_to_monster_id(object.unit_id, object.unit_id);
                self.add_object(unit);
            }
            &_ => ()
        }
    }

    fn handle_player_info(&mut self, parts: &[&str]) {

    }

    fn handle_ability_info(&mut self, parts: &[&str]) {
        let ability = parse::ability(parts);
        let interruptible_blockable = (ability.interruptible as u8) * 2 + (ability.blockable as u8);
        let buff = ESOLogsBuff {
            name: ability.name,
            damage_type: DamageType::None,
            id: ability.id,
            icon: ability.icon.strip_suffix(".png").map(|s| s.to_string()).unwrap_or_else(|| ability.icon),
            caused_by_id: 0,
            interruptible_blockable: interruptible_blockable,
        };
        self.add_buff(buff);
    }

    // get_buff_icon for unit icon
    fn handle_combat_event(&mut self, parts: &[&str]) {
        let source = parse::unit_state(parts, 9);
        let target = if parts[19] == "*" {
            source.clone()
        } else {
            parse::unit_state(parts, 19)
        };
        let source_equal_target = parts[19] == "*";
        let ability_id = parts[8].parse().unwrap();
        let buff_event= ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index: self.unit_index(source.unit_id).map_or(0, |idx| idx as u16),
            target_unit_index: self.unit_index(target.unit_id).map_or(0, |idx| idx as u16),
            buff_index: self.buff_index(ability_id).map_or(0, |idx| idx as u32),
        };
        self.add_buff_event(buff_event);
        let ev = event::Event {
            time: parts[0].parse().unwrap(),
            result: event::parse_event_result(parts[2]).unwrap(),
            damage_type: event::parse_damage_type(parts[3]),
            power_type: parts[4].parse().unwrap(),
            hit_value: parts[5].parse().unwrap(),
            overflow: parts[6].parse().unwrap(),
            cast_track_id: parts[7].parse().unwrap(),
            ability_id: parts[8].parse().unwrap(),
            source_unit_state: source,
            target_unit_state: target,
        };
        match ev.result {
            EventResult::Damage | EventResult::BlockedDamage | EventResult::DotTick | EventResult::CriticalDamage | EventResult::DotTickCritical => {
                let icon = self.eso_logs_log.get_buff_icon(ability_id);
                if let Some(unit_idx) = (buff_event.source_unit_index as usize).checked_sub(1) {
                    if unit_idx < self.eso_logs_log.units.len() {
                        let unit = &mut self.eso_logs_log.units[unit_idx];
                        if icon != "nil" && icon != "death_recap_melee_basic" {
                            unit.icon = Some(icon.clone());
                        } else if unit.icon.is_none() {
                            unit.icon = Some("death_recap_melee_basic".to_string());
                        }
                    }
                }
                self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: ev.time,
                        line_type: if source_equal_target {ESOLogsLineType::CastOnSelf} else {ESOLogsLineType::CastOnOthers},
                        buff_event: buff_event,
                        cast: ESOLogsCastBase {
                            magic_number_1: 16,
                            magic_number_2: 16,
                            cast_id_origin: ev.cast_track_id,
                            source_unit_state: ESOLogsUnitState {
                                unit_state: source,
                                magic_index: 0,
                            },
                            target_unit_state: ESOLogsUnitState {
                                unit_state: target,
                                magic_index: 0,
                            },
                        },
                        cast_information: Some(ESOLogsCastData {
                            critical: match ev.result {
                                EventResult::Damage | EventResult::BlockedDamage | EventResult::DotTick => 1,
                                EventResult::CriticalDamage | EventResult::DotTickCritical => 2,
                                _ => 0,
                            },
                            hit_value: if ev.hit_value == 0 {ev.overflow} else {ev.hit_value},
                            overflow: ev.overflow,
                        })
                    }
                ));
            }
            EventResult::HotTick | EventResult::CriticalHeal | EventResult::Heal | EventResult::HotTickCritical => {
                self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: ev.time,
                        line_type: if source_equal_target {ESOLogsLineType::CastOnSelf} else {ESOLogsLineType::CastOnOthers},
                        buff_event: buff_event,
                        cast: ESOLogsCastBase {
                            magic_number_1: 16,
                            magic_number_2: 16,
                            cast_id_origin: ev.cast_track_id,
                            source_unit_state: ESOLogsUnitState {
                                unit_state: source,
                                magic_index: 0,
                            },
                            target_unit_state: ESOLogsUnitState {
                                unit_state: target,
                                magic_index: 0,
                            },
                        },
                        cast_information: Some(ESOLogsCastData {
                            critical: match ev.result {
                                EventResult::HotTick | EventResult::Heal => 1,
                                EventResult::HotTickCritical | EventResult::CriticalHeal => 2,
                                _ => 0,
                            },
                            hit_value: if ev.hit_value == 0 {ev.overflow} else {ev.hit_value},
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
                            magic_number_1: 16,
                            magic_number_2: 16,
                            cast_id_origin: ev.cast_track_id,
                            source_unit_state: ESOLogsUnitState {
                                unit_state: source,
                                magic_index: 0,
                            },
                            target_unit_state: ESOLogsUnitState {
                                unit_state: target,
                                magic_index: 0,
                            },
                        },
                        hit_value: ev.hit_value,
                        overflow: ev.overflow,
                        resource_type: match ev.power_type {
                            0 => ESOLogsResourceType::Magicka,
                            1 => ESOLogsResourceType::Stamina,
                            8 => ESOLogsResourceType::Ultimate,
                            4 => ESOLogsResourceType::Health,
                            _ => {eprintln!("Unknown power type: {}", ev.power_type); ESOLogsResourceType::Health},
                        },
                    }
                ))
            }
            _ => (),
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
        let buff_event= ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index: self.unit_index(source.unit_id).map_or(0, |idx| idx as u16),
            target_unit_index: self.unit_index(target.unit_id).map_or(0, |idx| idx as u16),
            buff_index: self.buff_index(ability_id).map_or(0, |idx| idx as u32),
        };
        self.add_buff_event(buff_event);
    }

    fn handle_effect_changed(&mut self, parts: &[&str]) {
        let source = parse::unit_state(parts, 6);
        let target = if parts[16] == "*" {
            source.clone()
        } else {
            parse::unit_state(parts, 16)
        };
        let ability_id = parts[5].parse().unwrap();
        let mut buff_event= ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index: self.unit_index(source.unit_id).map_or(0, |idx| idx as u16),
            target_unit_index: self.unit_index(target.unit_id).map_or(0, |idx| idx as u16),
            buff_index: self.buff_index(ability_id).map_or(0, |idx| idx as u32),
        };
        if parts[2] == "GAINED" {
            let x = self.add_buff_event(buff_event);
            buff_event.unique_index = x;
            self.add_log_event(ESOLogsEvent::BuffLine (
                ESOLogsBuffLine {
                    timestamp: parts[0].parse().unwrap(),
                    line_type: ESOLogsLineType::BuffGained,
                    buff_event: buff_event,
                    magic_number_1: 16,
                    magic_number_2: 16,
                    magic_entry: None,
                    magic_entry_2: None,
                }
            ));
        } else if parts[2] == "FADED" {
            let x = self.add_buff_event(buff_event);
            buff_event.unique_index = x;
            self.add_log_event(ESOLogsEvent::BuffLine (
                ESOLogsBuffLine {
                    timestamp: parts[0].parse().unwrap(),
                    line_type: ESOLogsLineType::BuffFaded,
                    buff_event: buff_event,
                    magic_number_1: 16,
                    magic_number_2: 16,
                    magic_entry: None,
                    magic_entry_2: None,
                }
            ));
        }
    }

    fn handle_map_changed(&mut self, parts: &[&str]) {
        
    }

    fn handle_zone_changed(&mut self, parts: &[&str]) {

    }

    fn handle_trial_init(&mut self, parts: &[&str]) {

    }

    fn handle_trial_end(&mut self, parts: &[&str]) {

    }

    const HEALTH_RECOVERY_BUFF_ID: u32 = 61322;
    fn handle_health_recovery(&mut self, parts: &[&str]) {
        let health_recovery = ESOLogsBuff {
            name: "UseDatabaseName".to_string(),
            damage_type: DamageType::Poison,
            id: Self::HEALTH_RECOVERY_BUFF_ID,
            icon: "crafting_dom_beer_002".to_string(),
            caused_by_id: 0,
            interruptible_blockable: 0
        };
        self.add_buff(health_recovery);
        let source = parse::unit_state(parts, 3);
        let source_id = self.unit_index(source.unit_id).map_or(0, |idx| idx as u16);
        let buff_index = self.buff_index(Self::HEALTH_RECOVERY_BUFF_ID).map_or(0, |idx| idx as u32);
        let buff_event = ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index: source_id,
            target_unit_index: source_id,
            buff_index: buff_index,
        };
        self.add_buff_event(buff_event);
    }
}
