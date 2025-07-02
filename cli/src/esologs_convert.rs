use std::{error::Error, fs::File, io::{BufRead, BufReader, BufWriter}, path::Path};
use std::io::Write;
use parser::{event::{self, DamageType, EventResult}, parse::{self}, player::{Class, Race}, unit::Reaction};
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
        self.eso_logs_log.unit_index(unit_id)
    }

    pub fn buff_index(&self, buff_id: u32) -> Option<usize> {
        self.eso_logs_log.buff_index(buff_id)
    }

    pub fn add_log_event(&mut self, event: ESOLogsEvent) {
        self.eso_logs_log.add_log_event(event);
    }

    pub fn get_cp_for_unit(&self, unit_id: u32) -> u16 {
        self.eso_logs_log.get_cp_for_unit(unit_id)
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
        let bytes = line.as_bytes();

        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'[' => {
                    bracket_level += 1;
                    if i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                        i += 1;
                    }
                }
                b']' => {
                    if bracket_level > 0 {
                        bracket_level -= 1;
                    }
                    if i + 1 < bytes.len() && bytes[i + 1] == b']' {
                        i += 1;
                    }
                }
                b',' if bracket_level == 0 => {
                    let field = line[start..i].trim_matches(&['[', ']'][..]).trim();
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
            let field = line[start..].trim_matches(&['[', ']'][..]).trim();
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
            _ => {},
        }
    }

    fn handle_begin_log(&mut self, parts: Vec<&str>) {
        self.megaserver = parts[4].to_owned();
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
        // println!("Player info: {:?}", parts);
        let length = parts.len();
        if length < 8 {
            eprintln!("Invalid PLAYER_INFO line: {:?}", parts);
            return;
        }
        self.add_log_event(ESOLogsEvent::PlayerInfo(
            ESOLogsPlayerBuild {
                timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
                line_type: ESOLogsLineType::PlayerInfo,
                unit_id: parts[2].parse().unwrap(),
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
        let ability_id = parts[8].parse().unwrap();
        let mut buff_event= ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index: self.unit_index(source.unit_id).map_or(0, |idx| idx as u16),
            target_unit_index: self.unit_index(target.unit_id).map_or(0, |idx| idx as u16),
            buff_index: self.buff_index(ability_id).map_or(0, |idx| idx as u32),
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
            ability_id: parts[8].parse().unwrap(),
            source_unit_state: source,
            target_unit_state: target,
        };
        let critical = match ev.result {
                EventResult::Damage | EventResult::BlockedDamage | EventResult::DotTick => 1,
                EventResult::CriticalDamage | EventResult::DotTickCritical => 2,
                EventResult::HotTick | EventResult::Heal => 1,
                EventResult::HotTickCritical | EventResult::CriticalHeal => 2,
                _ => 0,
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
                        line_type: if &critical == &2 {ESOLogsLineType::CriticalDamage} else {ESOLogsLineType::Cast},
                        buff_event: buff_event,
                        cast: ESOLogsCastBase {
                            magic_number_1: 16,
                            magic_number_2: 64,
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
            }
            EventResult::HotTick | EventResult::CriticalHeal | EventResult::Heal | EventResult::HotTickCritical => {
                self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: ev.time,
                        line_type: match ev.result {
                            EventResult::HotTick | EventResult::HotTickCritical => {ESOLogsLineType::HotTick}
                            _ => {ESOLogsLineType::Heal}
                        },
                        buff_event: buff_event,
                        cast: ESOLogsCastBase {
                            magic_number_1: 16,
                            magic_number_2: 16,
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
        let mut buff_event= ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index: self.unit_index(source.unit_id).map_or(0, |idx| idx as u16),
            target_unit_index: self.unit_index(target.unit_id).map_or(0, |idx| idx as u16),
            buff_index: self.buff_index(ability_id).map_or(0, |idx| idx as u32),
        };
        let cast_id: u32 = parts[4].parse().unwrap();
        self.eso_logs_log.buffs_hashmap.insert(cast_id, buff_event.buff_index as usize);
        buff_event.unique_index = self.add_buff_event(buff_event);
        if cast_id != 0 {
            if let Some(buff) = self.eso_logs_log.buffs.get_mut(buff_event.buff_index as usize-1) {
                buff.caused_by_id = ability_id;
            }
        }
        let cast_time = parts[2].parse::<u32>().unwrap();
        self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
                        line_type: if cast_time > 0 {ESOLogsLineType::CastWithCastTime} else {ESOLogsLineType::Cast},
                        buff_event: buff_event,
                        cast: ESOLogsCastBase {
                            magic_number_1: 16,
                            magic_number_2: 16,
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
        let cast_id: u32 = parts[4].parse().unwrap();
        buff_event.unique_index = self.add_buff_event(buff_event);
        let mut index = None;
        if cast_id != 0 {
            index = self.eso_logs_log.buffs_hashmap.get(&cast_id).cloned();
            let caused_by_id = index.map_or(0, |idx| idx as u32);
            let buff2_id = if caused_by_id > 0 && (caused_by_id as usize) - 1 < self.eso_logs_log.buffs.len() {
                Some(self.eso_logs_log.buffs[(caused_by_id as usize) - 1].id)
            } else {
                None
            };
            if let Some(buff) = self.eso_logs_log.buffs.get_mut(buff_event.buff_index as usize) {
                if let Some(id) = buff2_id {
                    buff.caused_by_id = id;
                }
            }
        }
        if parts[2] == "GAINED" {
            self.add_log_event(ESOLogsEvent::BuffLine (
                ESOLogsBuffLine {
                    timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
                    line_type: ESOLogsLineType::BuffGained,
                    buff_event: buff_event,
                    magic_number_1: 16,
                    magic_number_2: 16,
                    source_cast_index: index,
                    source_shield: source.shield,
                    target_shield: target.shield,
                }
            ));
        } else if parts[2] == "FADED" {
            self.add_log_event(ESOLogsEvent::BuffLine (
                ESOLogsBuffLine {
                    timestamp: parts[0].parse::<u64>().unwrap() - self.timestamp_offset as u64,
                    line_type: ESOLogsLineType::BuffFaded,
                    buff_event: buff_event,
                    magic_number_1: 16,
                    magic_number_2: 16,
                    source_cast_index: None,
                    source_shield: source.shield,
                    target_shield: target.shield,
                }
            ));
        }
    }

    // 2960,MAP_CHANGED,1721,"Moon-Sugar Meadow","housing/moonsugarmeadow_base"
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

    // 5614990,END_TRIAL,15,5400266,T,164609,0
    // 5614983|55|15|5400266|1|164609
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
            let tbl_data = build_master_table(&elp);
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

fn build_report_segment(elp: &ESOLogProcessor) -> String {
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

fn build_master_table(elp: &ESOLogProcessor) -> String {
    let approx_capacity = (elp.eso_logs_log.units.len()
        + elp.eso_logs_log.buffs.len()
        + elp.eso_logs_log.effects.len())
        * 64;
    let mut out = String::with_capacity(approx_capacity);

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

    out.push_str("0\n");
    out
}