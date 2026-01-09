use std::{collections::{HashMap, HashSet}, error::Error, fs::File, io::{BufRead, BufReader, BufWriter}, path::Path, sync::{Arc, atomic::{AtomicBool, Ordering}}, u16};
use std::io::Write;
use esosim::{data::{critical_damage::LUCENT_ECHOES_ID, item_type::GearSlot, major_minor::SAVAGERY_MINOR_ID}, engine::character::Character, models::player::{ActiveBar, GearPiece}};
use parser::{EventType, UnitAddedEventType, effect::{self, StatusEffectType}, event::{self, CastEndReason, DamageType, EventResult, is_damage_event, parse_cast_end_reason}, parse::{self, gear_piece}, player::{Class, Race}, unit::{self, Reaction, UnitState, blank_unit_state}};
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

const SWAP_WEAPONS: u32 = 28541;
const SWAP_WEAPONS_FRONTBAR: u32 = 61874;
const SWAP_WEAPONS_BACKBAR: u32 = 61875;

pub struct ESOLogProcessor {
    pub eso_logs_log: ESOLogsLog,
    pub megaserver: Arc<str>,
    pub timestamp_offset: u64,
    temporary_damage_buffer: u32,
    last_known_timestamp: u64,
    last_death_events: HashMap<usize, u64>,
    last_interrupt: Option<u32>, // unit_id of last interrupted unit
    base_timestamp: Option<u64>,
    most_recent_begin_log_timestamp: Option<u64>,
    zone: Option<u16>,
    in_combat: bool,
}

impl Default for ESOLogProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ESOLogProcessor {
    pub fn new() -> Self {
        let mut log = ESOLogsLog::new();
        log.reserve_capacity(
            250,
            500,
            20_000,
            10_000,
            20,
        );
        let health_recovery = ESOLogsBuff {
            name: "UseDatabaseName".into(),
            damage_type: DamageType::Heal,
            status_type: StatusEffectType::None,
            id: Self::HEALTH_RECOVERY_BUFF_ID,
            icon: "crafting_dom_beer_002".into(),
            caused_by_id: 0,
            interruptible_blockable: 0
        };
        let crit_damage = ESOLogsBuff {
            name: "Critical Damage".into(),
            damage_type: DamageType::None,
            status_type: StatusEffectType::None,
            id: 512,
            icon: "internal/ability_internal_cyan".into(),
            caused_by_id: 0,
            interruptible_blockable: 0
        };
        let power = ESOLogsBuff {
            name: "Weapon & Spell Damage".into(),
            damage_type: DamageType::None,
            status_type: StatusEffectType::None,
            id: 513,
            icon: "internal/ability_internal_cyan".into(),
            caused_by_id: 0,
            interruptible_blockable: 0
        };
        let physical_resistance = ESOLogsBuff {
            name: "Physical Resistance".into(),
            damage_type: DamageType::None,
            status_type: StatusEffectType::None,
            id: 514,
            icon: "internal/ability_internal_cyan".into(),
            caused_by_id: 0,
            interruptible_blockable: 0
        };
        let spell_resistance = ESOLogsBuff {
            name: "Spell Resistance".into(),
            damage_type: DamageType::None,
            status_type: StatusEffectType::None,
            id: 515,
            icon: "internal/ability_internal_cyan".into(),
            caused_by_id: 0,
            interruptible_blockable: 0
        };
        let critical_chance = ESOLogsBuff {
            name: "Critical Chance".into(),
            damage_type: DamageType::None,
            status_type: StatusEffectType::None,
            id: 516,
            icon: "internal/ability_internal_cyan".into(),
            caused_by_id: 0,
            interruptible_blockable: 0
        };
        let penetration = ESOLogsBuff {
            name: "Penetration".into(),
            damage_type: DamageType::None,
            status_type: StatusEffectType::None,
            id: 517,
            icon: "internal/ability_internal_cyan".into(),
            caused_by_id: 0,
            interruptible_blockable: 0
        };
        log.add_buff(health_recovery);
        log.add_buff(crit_damage);
        log.add_buff(power);
        log.add_buff(physical_resistance);
        log.add_buff(spell_resistance);
        log.add_buff(critical_chance);
        log.add_buff(penetration);
        Self {
            eso_logs_log: log,
            megaserver: "Unknown".into(),
            timestamp_offset: 0,
            temporary_damage_buffer: 0,
            last_known_timestamp: 0,
            last_death_events: HashMap::new(),
            last_interrupt: None,
            base_timestamp: None,
            most_recent_begin_log_timestamp: None,
            zone: None,
            in_combat: false,
        }
    }

    pub fn reset(&mut self) {
        self.temporary_damage_buffer = 0;
        self.last_known_timestamp = 0;
        self.last_interrupt = None;
        self.last_death_events.clear();
    }

    pub fn add_unit(&mut self, unit: ESOLogsUnit) -> usize {
        self.eso_logs_log.add_unit(unit)
    }

    pub fn map_unit_id_to_monster_id(&mut self, unit_id: u32, unit: &ESOLogsUnit) {
        self.eso_logs_log.map_unit_id_to_monster_id(unit_id, unit)
    }

    pub fn add_object(&mut self, object: ESOLogsUnit) -> usize {
        self.eso_logs_log.add_object(object)
    }

    pub fn add_buff(&mut self, buff: ESOLogsBuff) -> bool {
        self.eso_logs_log.add_buff(buff)
    }

    pub fn add_buff_event(&mut self, buff_event: ESOLogsBuffEvent) -> usize {
        self.eso_logs_log.add_buff_event(buff_event)
    }

    pub fn unit_index(&self, unit_id: u32) -> Option<usize> {
        if unit_id == 0 {return Some(usize::MAX)}
        self.eso_logs_log.unit_index(&unit_id)
    }

    pub fn object_index(&self, object_id: Arc<str>) -> Option<usize> {
        self.eso_logs_log.object_index(object_id)
    }

    pub fn buff_index(&self, buff_id: u32) -> Option<usize> {
        if buff_id == 0 {return Some(usize::MAX)}
        self.eso_logs_log.buff_index(buff_id)
    }

    pub fn add_log_event(&mut self, event: ESOLogsEvent) {
        self.eso_logs_log.add_log_event(event);
    }

    pub fn remove_overabundant_events(&mut self) {
        let mut buff_history: HashMap<(usize, usize), Vec<(usize, u64, bool)>> = HashMap::new();
        let mut first_source_target: HashMap<usize, (usize, usize)> = HashMap::new();

        for (idx, event) in self.eso_logs_log.events.iter().enumerate() {
            let ESOLogsEvent::BuffLine(line) = event else { continue };
            let is_gained = matches!(line.line_type, ESOLogsLineType::BuffGainedAlly | ESOLogsLineType::BuffGainedEnemy);
            let is_faded = matches!(line.line_type, ESOLogsLineType::BuffFadedAlly | ESOLogsLineType::BuffFadedEnemy);
            if !is_gained && !is_faded {continue;}

            first_source_target.entry(line.buff_event.buff_index).or_insert((line.buff_event.source_unit_index, line.buff_event.target_unit_index));

            buff_history.entry((line.buff_event.buff_index, line.buff_event.target_unit_index))
                .or_default()
                .push((idx, line.timestamp, is_gained));
        }

        let mut remove = HashSet::new();
        for entries in buff_history.values_mut() {
            entries.sort_by_key(|e| e.1);

            for w in entries.windows(2) {
                let (idx_a, ts_a, gained_a) = w[0];
                let (idx_b, ts_b, gained_b) = w[1];

                if !gained_a && gained_b && ts_b - ts_a <= 4 {
                    if let Some(ESOLogsEvent::BuffLine(line)) = self.eso_logs_log.events.get(idx_a) {
                        if let Some(buff) = self.eso_logs_log.buffs.get(line.buff_event.buff_index) {
                            if matches!(buff.id, 61898 | 61666 | 88490 | 88509 | 172621) {
                                remove.insert(idx_a);
                                remove.insert(idx_b);
                            }
                        }
                    }
                }
            }
        }


        if !remove.is_empty() {
            let mut i = 0;
            self.eso_logs_log.events.retain(|_| {
                let keep = !remove.contains(&i);
                i += 1;
                keep
            });
        }

        let mut updates = Vec::new();
        for (idx, event) in self.eso_logs_log.events.iter().enumerate() {
            let ESOLogsEvent::BuffLine(line) = event else { continue };
            let buff = match self.eso_logs_log.buffs.get(line.buff_event.buff_index) {
                Some(b) => b,
                None => continue,
            };

            let mut new_buff_event = line.buff_event.clone();

            if matches!(buff.id, 61898 | 61666 | 88490 | 88509 | 172621) {
                new_buff_event.source_unit_index = new_buff_event.target_unit_index;
            }

            updates.push((idx, new_buff_event));
        }

        for (idx, new_buff_event) in updates {
            let unique_index = self.add_buff_event(new_buff_event.clone());
            if let Some(ESOLogsEvent::BuffLine(line)) = self.eso_logs_log.events.get_mut(idx) {
                line.buff_event = new_buff_event;
                line.buff_event.unique_index = unique_index;
            }
        }
    }

    fn maybe_create_buff_event(&mut self, target_unit_id: u32, buff_index_id: u32) -> Result<Option<ESOLogsBuffEvent>, String> {
        if !self.eso_logs_log.esosim_characters.contains_key(&target_unit_id) {
            return Ok(None);
        }

        let source_unit_index = self.unit_index(target_unit_id).ok_or_else(|| format!("source_unit_index {} is out of bounds", target_unit_id))?;
        let target_unit_index = self.unit_index(target_unit_id).ok_or_else(|| format!("target_unit_index {} is out of bounds", target_unit_id))?;
        let buff_index = self.buff_index(buff_index_id).ok_or_else(|| format!("buff_index {} is out of bounds", buff_index_id))?;


        let mut event = ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index,
            target_unit_index,
            buff_index,
        };
        event.unique_index = self.add_buff_event(event);
        Ok(Some(event))
    }

    pub fn process_target_stats(&mut self, target: u32, target_allegiance: u8) -> Result<(), String> {
        let buff_event_crit = self.maybe_create_buff_event(target, 512)?;
        let buff_event_power = self.maybe_create_buff_event(target, 513)?;
        let buff_event_physical_resistance = self.maybe_create_buff_event(target, 514)?;
        let buff_event_spell_resistance = self.maybe_create_buff_event(target, 515)?;
        let buff_event_crit_chance = self.maybe_create_buff_event(target, 516)?;
        let buff_event_penetration = self.maybe_create_buff_event(target, 517)?;

        let stats_opt = {
            match self.eso_logs_log.esosim_characters.get_mut(&target) {
                Some(character) => {
                    let crit_damage = character.get_critical_damage_uncapped() as u32;
                    let power = character.get_power();
                    let armour_physical = character.get_armour(&esosim::models::damage::DamageType::PHYSICAL);
                    let armour_magic = character.get_armour(&esosim::models::damage::DamageType::MAGIC);
                    let crit_chance = character.get_critical_chance_raw();
                    let penetration = character.get_penetration();
                    Some((crit_damage, power, armour_physical, armour_magic, crit_chance, penetration))
                }
                None => None,
            }
        };

        let (crit_damage, power, armour_physical, armour_magic, crit_chance, penetration) =
            match stats_opt {
                Some(tuple) => tuple,
                None => return Ok(()),
            };

        macro_rules! maybe_update {
            ($buff_event:expr, $stacks:expr, $map:expr) => {
                if let Some(buff_event) = $buff_event {
                    let current = $map.get(&target).copied().unwrap_or(0u32);
                    let new_stacks_u32: u32 = $stacks;
                    if current != new_stacks_u32 {
                        self.add_log_event(ESOLogsEvent::StackUpdate(
                            ESOLogsBuffStacks {
                                timestamp: self.last_known_timestamp,
                                line_type: ESOLogsLineType::BuffStacksUpdatedAlly,
                                buff_event,
                                unit_instance_id: (0, 0),
                                source_allegiance: target_allegiance,
                                target_allegiance,
                                stacks: new_stacks_u32.try_into().unwrap_or(u16::MAX),
                            },
                        ));
                        $map.insert(target, new_stacks_u32);
                    }
                }
            };
        }

        maybe_update!(buff_event_crit, crit_damage, &mut self.eso_logs_log.critical_damage_done);
        maybe_update!(buff_event_power, power, &mut self.eso_logs_log.power);
        maybe_update!(buff_event_physical_resistance, armour_physical, &mut self.eso_logs_log.armour_physical);
        maybe_update!(buff_event_spell_resistance, armour_magic, &mut self.eso_logs_log.armour_spell);
        maybe_update!(buff_event_crit_chance, crit_chance, &mut self.eso_logs_log.crit_chance);
        maybe_update!(buff_event_penetration, penetration, &mut self.eso_logs_log.penetration);

        Ok(())
    }

    pub fn emit_all_target_stat_events(&mut self, target: u32, target_allegiance: u8) -> Result<(), String> {
        let buff_event_crit = self.maybe_create_buff_event(target, 512)?;
        let buff_event_power = self.maybe_create_buff_event(target, 513)?;
        let buff_event_physical_resistance = self.maybe_create_buff_event(target, 514)?;
        let buff_event_spell_resistance = self.maybe_create_buff_event(target, 515)?;
        let buff_event_crit_chance = self.maybe_create_buff_event(target, 516)?;
        let buff_event_penetration = self.maybe_create_buff_event(target, 517)?;

        let stats = {
            let character = match self
                .eso_logs_log
                .esosim_characters
                .get_mut(&target)
            {
                Some(c) => c,
                None => return Ok(()),
            };

            (
                character.get_critical_damage_uncapped() as u32,
                character.get_power(),
                character.get_armour(&esosim::models::damage::DamageType::PHYSICAL),
                character.get_armour(&esosim::models::damage::DamageType::MAGIC),
                character.get_critical_chance_raw(),
                character.get_penetration(),
            )
        };

        let (
            crit_damage,
            power,
            armour_physical,
            armour_magic,
            crit_chance,
            penetration,
        ) = stats;

        macro_rules! emit {
            ($buff_event:expr, $stacks:expr) => {
                if let Some(buff_event) = $buff_event {
                    self.add_log_event(ESOLogsEvent::StackUpdate(
                        ESOLogsBuffStacks {
                            timestamp: self.last_known_timestamp,
                            line_type: ESOLogsLineType::BuffStacksUpdatedAlly,
                            buff_event,
                            unit_instance_id: (0, 0),
                            source_allegiance: target_allegiance,
                            target_allegiance,
                            stacks: $stacks.try_into().unwrap_or(u16::MAX),
                        },
                    ));
                }
            };
        }

        emit!(buff_event_crit, crit_damage);
        emit!(buff_event_power, power);
        emit!(buff_event_physical_resistance, armour_physical);
        emit!(buff_event_spell_resistance, armour_magic);
        emit!(buff_event_crit_chance, crit_chance);
        emit!(buff_event_penetration, penetration);

        Ok(())
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
        reaction.unwrap_or(Reaction::None)
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
                    log::warn!("Error reading line: {e}");
                    continue;
                }
            };
            let modified_line = handle_line(line, &mut custom_log_data);
            for line in modified_line {
                self.handle_line(line);
            }
            lines += 1;
            if lines % 250_000 == 0 {
                log::info!("Processed {lines} lines");
                log::info!("Length of stuff: buffs:{}, effects:{}, units:{}, lines:{}", self.eso_logs_log.buffs.len(), self.eso_logs_log.effects.len(), self.eso_logs_log.units.len(), self.eso_logs_log.events.len());
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

        let r = match event {
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
            EventType::UnitRemoved   => Ok(()),
            EventType::Unknown       => {log::debug!("Unknown log line:\n{parts:?}"); Ok(())}
        };
        match r {
            Ok(()) => {},
            Err(e) => {log::debug!("Parts: {parts:?}"); log::debug!("Error: {e}")},
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

    fn handle_begin_log(&mut self, parts: &[String]) -> Result<(), String> {
        self.megaserver = parts[4].to_owned().into();

        let log_ts = parts[2]
            .parse::<u64>()
            .map_err(|e| format!("Failed to parse log_ts: {e}"))?;

        let rel_ticks = parts[0]
            .parse::<u64>()
            .map_err(|e| format!("Failed to parse rel_ticks: {e}"))?;

        self.most_recent_begin_log_timestamp = Some(log_ts);
        self.timestamp_offset = rel_ticks;

        if self.base_timestamp.is_none() {
            self.base_timestamp = Some(log_ts);
        }

        Ok(())
    }

    fn handle_end_combat(&mut self, parts: &[String]) -> Result<(), String> {
        let rel_ticks = parts[0]
            .parse::<u64>()
            .map_err(|e| format!("Failed to parse end combat timestamp: {e}"))?;

        let timestamp = self.calculate_timestamp(rel_ticks);

        self.add_log_event(ESOLogsEvent::EndCombat(
            ESOLogsCombatEvent {
                timestamp,
                line_type: ESOLogsLineType::EndCombat,
            }
        ));

        self.in_combat = false;
        
        self.eso_logs_log.current_health.clear();
        self.eso_logs_log.fight_units.clear();
        self.eso_logs_log.unit_index_during_fight.clear();

        Ok(())
    }

    fn reset_custom_stats(&mut self) {
        self.eso_logs_log.critical_damage_done.clear();
        self.eso_logs_log.power.clear();
        self.eso_logs_log.armour_physical.clear();
        self.eso_logs_log.armour_spell.clear();
        self.eso_logs_log.crit_chance.clear();
        self.eso_logs_log.penetration.clear();
    }

    fn handle_begin_combat(&mut self, parts: &[String]) -> Result<(), String> {
        let rel_ticks = parts[0]
            .parse::<u64>()
            .map_err(|e| format!("Failed to parse begin combat timestamp: {e}"))?;

        let timestamp = self.calculate_timestamp(rel_ticks);

        self.add_log_event(ESOLogsEvent::BeginCombat(
            ESOLogsCombatEvent {
                timestamp,
                line_type: ESOLogsLineType::BeginCombat,
            }
        ));

        self.in_combat = true;
        self.reset_custom_stats();

        Ok(())
    }

    const BOSS_CLASS_ID: u8 = 100;
    const PET_CLASS_ID: u8 = 50;
    const OBJECT_CLASS_ID: u8 = 0;
    fn handle_unit_added(&mut self, parts: &[String]) -> Result<(), String> {
        let event = parts.get(3)
        .ok_or("Missing field at index 3 for UnitAddedEventType")?
        .as_str()
        .into();
        match event {
            UnitAddedEventType::Player => {
                let player = parse::player(parts);

                let mut unit = ESOLogsUnit {
                    name: player.name.trim_matches('"').into(),
                    player_data: Some(ESOLogsPlayerSpecificData {
                        username: player.display_name.into(),
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

                if unit.name.is_empty() && player.is_grouped_with_local_player {
                    let id = if player.champion_points > 0 {player.champion_points} else {player.level.into()};
                    unit.name = format!("Anonymous {id}").into();
                }

                self.map_unit_id_to_monster_id(player.unit_id, &unit);
                self.eso_logs_log.shield_values.insert(player.unit_id, 0);
                if let Some(unit_index) = self.eso_logs_log.session_id_to_units_index.get(&unit.unit_id) {
                    if let Some(player) = self.eso_logs_log.units.get_mut(*unit_index) {
                        if player.name == "Offline".into() {
                            *player = unit.clone();
                        }
                    }
                }
                self.eso_logs_log.players.insert(player.unit_id, true);
                self.eso_logs_log.players.insert(player.player_per_session_id, true);
                let index = self.add_unit(unit);
                self.eso_logs_log.unit_id_to_units_index.insert(player.unit_id, index);
                self.eso_logs_log.esosim_characters.insert(player.unit_id, Character::new(player.unit_id.clone()));
            }
            UnitAddedEventType::Monster => {
                let monster = parse::monster(parts);
                let unit = ESOLogsUnit {
                    name: monster.name.trim_matches('"').into(),
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
                let pet_owner_index = if monster.owner_unit_id != 0 {
                    Some(self.unit_index(monster.owner_unit_id)
                        .ok_or_else(|| format!("Pet owner unit {} not found", monster.owner_unit_id))?)
                } else {
                    None
                };
                self.map_unit_id_to_monster_id(monster.unit_id, &unit);
                if monster.is_boss {
                    self.eso_logs_log.bosses.insert(unit.unit_id, true);
                    self.eso_logs_log.bosses.insert(monster.unit_id, true);
                }
                let index = self.add_unit(unit);
                // if monster.name.trim_matches('"').to_lowercase().contains("wamasu") {log::debug!("{:?} has index {index}", monster);}
                self.eso_logs_log.unit_id_to_units_index.insert(monster.unit_id, index);
                self.eso_logs_log.shield_values.insert(monster.unit_id, 0);
                if pet_owner_index.is_some() {
                    // log::debug!("{} - New unit {} ({}, {}) with index {} belonging to {}", parts[0], monster.name.trim_matches('"'), monster.monster_id, monster.unit_id, index, self.eso_logs_log.units[pet_owner_index.unwrap()].name);
                    let pet_relationship = ESOLogsPetRelationship {
                        owner_index: pet_owner_index.ok_or_else(|| "Failed to parse owner_index".to_string())?,
                        pet: ESOLogsPet { pet_type_index: index }
                    };
                    if !self.eso_logs_log.pets.iter().any(|rel| rel.pet.pet_type_index == pet_relationship.pet.pet_type_index) {
                        log::debug!("{}, Pet relationship: {} for unit: {} ({}), owner: {}, due to unit id {}", parts[0], pet_relationship, monster.name.trim_matches('"'), monster.monster_id, self.eso_logs_log.units[pet_relationship.owner_index].name, monster.unit_id);
                        self.eso_logs_log.pets.push(pet_relationship);
                    }
                }
            }
            UnitAddedEventType::Object | UnitAddedEventType::SiegeWeapon => {
                let object = parse::object(parts);
                let unit = ESOLogsUnit {
                    name: object.name.trim_matches('"').into(),
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
        Ok(())
    }

    fn handle_player_info(&mut self, parts: &[String]) -> Result<(), String> {
        let length = parts.len();
        if length < 8 {
            log::warn!("Invalid PLAYER_INFO line: {parts:?}");
            return Err("Player info definition too short".to_string());
        }

        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().map_err(|e| format!("Failed to parse player_info timestamp {e}"))?);

        let gear_pieces: Vec<Option<(GearPiece, GearSlot)>> = parts[5..length-2].iter().map(|s| gear_piece(s)).collect();
        let primary_skills: Vec<u32> = parts[length-2].split(',').map(|s| s.parse::<u32>().unwrap()).collect();
        let backup_skills: Vec<u32> = parts[length-1].split(',').map(|s| s.parse::<u32>().unwrap()).collect();
        let player_id = parts[2].parse().map_err(|e| format!("Failed to parse player parts[2]: {e}"))?;

        let long_term_buffs: Vec<u32> = parts[3].split(',').map(|x| x.parse::<u32>().unwrap_or_default()).collect();
        let long_term_buff_stacks: Vec<u8> = parts[4].split(',').map(|x| x.parse::<u8>().unwrap_or_default()).collect();
        if let Some(player) = self.eso_logs_log.esosim_characters.get_mut(&player_id) {
            for gear in gear_pieces {
                if let Some((real_gear, gear_slot)) = gear {
                    player.set_gear_piece(&gear_slot, real_gear);
                }
            }
            player.set_skills_on_bar(&ActiveBar::Primary, primary_skills);
            player.set_skills_on_bar(&ActiveBar::Backup, backup_skills);

            for buff in &long_term_buffs {
                player.add_buff(*buff, 1);
            }
        }

        self.add_log_event(ESOLogsEvent::PlayerInfo(
            ESOLogsPlayerBuild {
                timestamp,
                line_type: ESOLogsLineType::PlayerInfo,
                unit_index: self.unit_index(player_id).ok_or_else(|| "Failed to unwrap player unit_index".to_string())?,
                permanent_buffs: long_term_buffs.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",").into(),
                buff_stacks: long_term_buff_stacks.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",").into(),
                gear: parts[5..length-2].iter().map(|s| s.to_string()).collect(),
                primary_abilities: parts[length-2].to_string().into(),
                backup_abilities: parts[length-1].to_string().into(),
            }
        ));
        Ok(())
    }

    fn handle_ability_info(&mut self, parts: &[String]) -> Result<(), String> {
        let ability = parse::ability(parts);
        let interruptible_blockable = (ability.interruptible as u8) * 2 + (ability.blockable as u8);
        let damage_type = DamageType::None;
        let buff = ESOLogsBuff {
            name: ability.name,
            damage_type,
            status_type: StatusEffectType::None,
            id: ability.id,
            icon: ability.icon.strip_suffix(".png").map(|s| s.into()).unwrap_or_else(|| ability.icon),
            caused_by_id: 0,
            interruptible_blockable,
        };
        self.add_buff(buff);
        Ok(())
    }

    fn update_shield_history(esolog: &mut ESOLogsLog, unit_id: u32, shield: u32, buff_event: &ESOLogsBuffEventKey2) {
        let units_stored_shield = *esolog.shield_values.get(&unit_id).unwrap_or(&0);
        let buff = &esolog.buffs[buff_event.buff_index];
        if shield != units_stored_shield || buff.id == 146311 /* frost safeguard */ {
            // log::trace!("Comparing shields for unit {}: {} new vs stored {}", unit_id, shield, units_stored_shield);
            if let Some(shield_buffs_for_unit) = esolog.shields.get_mut(&unit_id) {
                shield_buffs_for_unit.insert(buff_event.buff_index, buff_event.clone());
                // log::trace!("Adding buff index: {}", buff_event.buff_index);
            } else {
                let mut hashmap = HashMap::new();
                hashmap.insert(buff_event.buff_index, buff_event.clone());
                esolog.shields.insert(unit_id, hashmap);
                // log::trace!("Adding buff index: {}", buff_event.buff_index);
            }
            esolog.shield_values.insert(unit_id, shield);
        }
    }

    fn handle_combat_event(&mut self, parts: &[String]) -> Result<(), String> {
        let source = parse::unit_state(parts, 9);
        let target = if parts[19] == "*" {
            source
        } else {
            parse::unit_state(parts, 19)
        };
        let result = event::parse_event_result(&parts[2]).ok_or_else(|| "Failed to parse combat event_result".to_string())?;
        if is_damage_event(result) && !self.in_combat {return Ok(())}
        let mut ability_id = parts[8].parse().map_err(|e| format!("Failed to parse ability_id: {e}"))?;
        if ability_id == 0 && result == EventResult::SoulGemResurrectionAccepted {ability_id = 26770}
        let mut buff_event= ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index: self.unit_index(source.unit_id).ok_or_else(|| format!("source_unit_index {} is out of bounds", source.unit_id))?,
            target_unit_index: self.unit_index(target.unit_id).ok_or_else(|| format!("target_unit_index {} is out of bounds", target.unit_id))?,
            buff_index: self.buff_index(ability_id).ok_or_else(|| format!("buff_index {ability_id} is out of bounds"))?,
        };
        let unique_index = self.add_buff_event(buff_event);
        buff_event.unique_index = unique_index;
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().map_err(|e| format!("Failed to parse timestamp: {e}"))?);
        let ev = event::Event {
            time: timestamp,
            result,
            damage_type: event::parse_damage_type(&parts[3]),
            power_type: parts[4].parse().map_err(|e| format!("Failed to parse power_type: {e}"))?,
            hit_value: parts[5].parse().map_err(|e| format!("Failed to parse hit_value: {e}"))?,
            overflow: parts[6].parse().map_err(|e| format!("Failed to parse overflow: {e}"))?,
            cast_track_id: parts[7].parse().map_err(|e| format!("Failed to parse cast_track_id: {e}"))?,
            ability_id,
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
        let dont_skip_enemy_id = if let Some(t) = self.eso_logs_log.units.get(buff_event.target_unit_index) {
            match t.unit_id {
                113566 => false, // archwizard twelvane
                121166 => false, // count ryelaz
                121469 => false, // zilyesset
                _ => true,
            }
        } else {
            true
        };
        let should_add_death_event = matches!(ev.result, EventResult::Damage | EventResult::DotTick | EventResult::CriticalDamage | EventResult::DotTickCritical | EventResult::BlockedDamage) && target.health == 0 && dont_skip_enemy_id;

        match ev.result {
            EventResult::DamageShielded => {
                if ev.hit_value == 0 {return Ok(())};
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
                                        // log::trace!("shield error: {:?}", shield_buff_event);
                                        // log::trace!("shield parts: {:?}", parts);
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
            EventResult::Dodged => {
                if let Some(buff) = self.eso_logs_log.buffs.get_mut(buff_event.buff_index) {
                    if ev.damage_type != DamageType::Heal { // You can't dodge a heal
                        buff.damage_type = ev.damage_type;
                    }
                }
                let instance_ids = (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0));
                self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: timestamp,
                        line_type: ESOLogsLineType::Damage,
                        buff_event,
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
                // if self.in_combat == false {return Ok(())}
                if ev.hit_value == 0 && ev.overflow == 0 {return Ok(())}
                if !self.eso_logs_log.bosses.contains_key(&source.unit_id)  && buff_event.source_unit_index < self.eso_logs_log.units.len() {
                    let unit = &mut self.eso_logs_log.units[buff_event.source_unit_index];
                    if icon != "nil".into() && icon != "death_recap_melee_basic".into() {
                        unit.icon = Some(icon.clone());
                    } else if unit.icon.is_none() {
                        unit.icon = Some("death_recap_melee_basic".into());
                    }
                }
                if buff_event.target_unit_index < self.eso_logs_log.units.len() {
                    let unit = &mut self.eso_logs_log.units[buff_event.target_unit_index];
                    if unit.icon.is_none() {
                        unit.icon = Some("death_recap_melee_basic".into());
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
                        timestamp: timestamp,
                        line_type: match ev.result {
                            EventResult::Damage | EventResult::CriticalDamage | EventResult::BlockedDamage => ESOLogsLineType::Damage,
                            _ => ESOLogsLineType::DotTick,
                        },
                        buff_event,
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
                            blocked: ev.result == EventResult::Blocked,
                        })
                    }
                ));
                if should_add_death_event {
                    if timestamp - self.last_death_events.get(&buff_event.target_unit_index).unwrap_or(&0) < 3000 {return Ok(())}
                    self.last_death_events.insert(buff_event.target_unit_index, timestamp);
                    if source_allegiance == 32 {source_allegiance = 64} // if it's not an enemy then it is a friend (edge case)
                    self.add_log_event(ESOLogsEvent::CastLine(
                        ESOLogsCastLine {
                            timestamp,
                            line_type: ESOLogsLineType::Death,
                            buff_event,
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
                    ));
                    return Ok(());
                }
            }
            EventResult::HotTick | EventResult::CriticalHeal | EventResult::Heal | EventResult::HotTickCritical => {
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
                        buff_event,
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
            EventResult::PowerEnergize | EventResult::PowerDrain => {
                self.add_log_event(ESOLogsEvent::PowerEnergize(
                    ESOLogsPowerEnergize {
                        timestamp: timestamp,
                        line_type: ESOLogsLineType::PowerEnergize,
                        buff_event,
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
                        hit_value: if ev.result == EventResult::PowerEnergize {ev.hit_value.try_into().unwrap()} else {-TryInto::<i32>::try_into(ev.hit_value).unwrap()},
                        overflow: ev.overflow,
                        resource_type: match ev.power_type {
                            1 => ESOLogsResourceType::Magicka,
                            4 => ESOLogsResourceType::Stamina,
                            8 => ESOLogsResourceType::Ultimate,
                            _ => {log::warn!("Unknown power type: {}", ev.power_type); ESOLogsResourceType::Health},
                        },
                    }
                ));
                return Ok(())
            }
            EventResult::Immune => {
                let instance_ids = (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0));
                self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp: timestamp,
                        line_type: ESOLogsLineType::Damage,
                        buff_event,
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
                            override_magic_number: Some(10), // magic number that just works based on observed log uploads. todo: figure out 1-9, 11-??
                            replace_hitvalue_overflow: false,
                            blocked: false,
                        })
                    }
                ));
            }
            EventResult::Interrupt => {
                self.last_interrupt = Some(target.unit_id);
            }
            EventResult::SoulGemResurrectionAccepted => {
                self.last_death_events.insert(buff_event.target_unit_index, 0); // reset death event cooldown so that if they instantly die, get ressed, and then die it is fine. todo: check if works with necro res.
                self.add_log_event(ESOLogsEvent::CastLine(
                    ESOLogsCastLine {
                        timestamp,
                        line_type: ESOLogsLineType::Resurrect,
                        buff_event,
                        unit_instance_id: (0, 0), // can only ever resurrect a player or companion, who should always have instance id of 0
                        cast: ESOLogsCastBase {
                            source_allegiance,
                            target_allegiance,
                            cast_id_origin: 0,
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
                if source_buff.caused_by_id == source_buff.id {
                    target_buff.caused_by_id = source_buff.id;
                }
            } else if caused_by_idx == target_idx {
                if let Some(buff) = self.eso_logs_log.buffs.get_mut(target_idx) {
                    buff.caused_by_id = buff.id;
                }
            }
        };
        Ok(())
    }

    fn handle_begin_cast(&mut self, parts: &[String]) -> Result<(), String> {
        let source = parse::unit_state(parts, 6);
        let target = if parts[16] == "*" {
            source
        } else {
            parse::unit_state(parts, 16)
        };
        let ability_id = parts[5].parse().map_err(|e| format!("Failed to parse ability_id: {e}"))?;
        if let Some(character) = self.eso_logs_log.esosim_characters.get_mut(&source.unit_id) {
            match ability_id {
                SWAP_WEAPONS => character.swap_bars(None),
                SWAP_WEAPONS_FRONTBAR => character.swap_bars(Some(&ActiveBar::Primary)),
                SWAP_WEAPONS_BACKBAR => character.swap_bars(Some(&ActiveBar::Backup)),
                _ => {
                    if let Some(bar) = character.get_bar_of_skill_id(&ability_id).cloned() {
                        character.swap_bars(Some(&bar));
                    }
                }
            }
        }
        let cast_time = parts[2].parse::<u32>().map_err(|e| format!("Failed to parse cast_time: {e}"))?;
        let cast_track_id = parts[4].parse::<u32>().map_err(|e| format!("Failed to parse cast_track_id: {e}"))?;
        if cast_time > 0 {
            self.eso_logs_log.cast_with_cast_time.insert(cast_track_id);
        }
        let mut buff_event= ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index: self.unit_index(source.unit_id).ok_or_else(|| format!("source_unit_index {} is out of bounds", source.unit_id))?,
            target_unit_index: self.unit_index(target.unit_id).ok_or_else(|| format!("target_unit_index {} is out of bounds", target.unit_id))?,
            buff_index: self.buff_index(ability_id).ok_or_else(|| format!("buff_index {ability_id} is out of bounds"))?,
        };
        let cast_id: u32 = parts[4].parse().map_err(|e| format!("Failed to parse cast_id: {e}"))?;
        buff_event.unique_index = self.add_buff_event(buff_event);
        if cast_id != 0 {
            self.eso_logs_log.buffs_hashmap.insert(cast_id, buff_event.buff_index);
            self.eso_logs_log.cast_id_hashmap.insert(cast_id, buff_event.unique_index);
            if let Some(buff) = self.eso_logs_log.buffs.get_mut(buff_event.buff_index) {
                buff.caused_by_id = ability_id;
            }
        }
        let source_allegiance = Self::allegiance_from_reaction(self.allegiance_from_unit_state(source));
        let target_allegiance = Self::allegiance_from_reaction(self.allegiance_from_unit_state(target));
        let instance_ids = (self.index_in_session(source.unit_id).unwrap_or(0), self.index_in_session(target.unit_id).unwrap_or(0));
        self.eso_logs_log.cast_id_source_unit_id.insert(cast_id, source.unit_id);
        self.eso_logs_log.cast_id_target_unit_id.insert(cast_id, target.unit_id);
        if self.eso_logs_log.bosses.contains_key(&source.unit_id)
            && source == target {
                if let Some(buff) = self.eso_logs_log.buffs.get(buff_event.buff_index) {
                    if let Some(unit_index) = self.eso_logs_log.unit_index(&source.unit_id) {
                        if let Some(unit) = self.eso_logs_log.units.get_mut(unit_index) {
                            unit.icon = Some(buff.icon.clone());
                        }
                    }
                }
            }
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().map_err(|e| format!("Failed to parse timestamp: {e}"))?);
        self.last_known_timestamp = timestamp;
        self.add_log_event(ESOLogsEvent::CastLine(
            ESOLogsCastLine {
                timestamp: self.last_known_timestamp,
                line_type: if cast_time > 0 {ESOLogsLineType::CastWithCastTime} else {ESOLogsLineType::Cast},
                buff_event,
                unit_instance_id: instance_ids,
                cast: ESOLogsCastBase {
                    source_allegiance,
                    target_allegiance,
                    cast_id_origin: parts[4].parse().map_err(|e| format!("Failed to parse cast_id_origin: {e}"))?,
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
        Ok(())
    }

    fn handle_effect_changed(&mut self, parts: &[String]) -> Result<(), String> {
        let source = parse::unit_state(parts, 6);
        if parts.len() == 16 {log::error!("{parts:?}")}
        let target_equal_source = parts[16] == "*";
        let target = if target_equal_source {
            source
        } else {
            parse::unit_state(parts, 16)
        };
        let ability_id = parts[5].parse().map_err(|e| format!("Failed to parse timestamp: {e}"))?;
        let mut buff_event= ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index: self.unit_index(source.unit_id).ok_or_else(|| format!("source_unit_index {} is out of bounds", source.unit_id))?,
            target_unit_index: self.unit_index(target.unit_id).ok_or_else(|| format!("target_unit_index {} is out of bounds", target.unit_id))?,
            buff_index: self.buff_index(ability_id).ok_or_else(|| format!("buff_index {ability_id} is out of bounds"))?,
        };
        let cast_id: u32 = parts[4].parse().map_err(|e| format!("Failed to parse cast_id: {e}"))?;
        buff_event.unique_index = self.add_buff_event(buff_event);
        let index = self.eso_logs_log.buffs_hashmap.get(&cast_id).copied();
        if let Some(caused_by_idx_raw) = index {
            let caused_by_idx = caused_by_idx_raw;
            let target_idx = buff_event.buff_index;
            if let Some((target_buff, source_buff)) = self.buffs_pair_mut(target_idx, caused_by_idx) {
                if source_buff.caused_by_id == source_buff.id {
                    target_buff.caused_by_id = source_buff.id;
                }
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
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().map_err(|e| format!("Failed to parse timestamp: {e}"))?);
        self.last_known_timestamp = timestamp;
        let stacks = parts[3].parse::<u16>().unwrap_or(1);
        if parts[2] == "GAINED" {
            if let Some(character) = self.eso_logs_log.esosim_characters.get_mut(&target.unit_id) {
                match ability_id {
                    LUCENT_ECHOES_ID => if source.unit_id == target.unit_id {} else {character.add_buff(ability_id.clone(), stacks as u8)},
                    _ => character.add_buff(ability_id.clone(), stacks as u8),
                }
            }
            self.add_log_event(ESOLogsEvent::BuffLine (
                ESOLogsBuffLine {
                    timestamp: self.last_known_timestamp,
                    line_type: if source_allegiance == 16 && target_allegiance == 16 {ESOLogsLineType::BuffGainedAlly} else {ESOLogsLineType::BuffGainedEnemy},
                    buff_event,
                    unit_instance_id: instance_ids,
                    source_allegiance,
                    target_allegiance,
                    source_cast_index: index,
                    source_shield: source.shield,
                    target_shield: target.shield,
                }
            ));
        } else if parts[2] == "FADED" {
            if let Some(character) = self.eso_logs_log.esosim_characters.get_mut(&target.unit_id) {
                match ability_id {
                    SAVAGERY_MINOR_ID => {},
                    _ => character.remove_buff(ability_id.clone()),
                }
            }
            self.add_log_event(ESOLogsEvent::BuffLine (
                ESOLogsBuffLine {
                    timestamp: self.last_known_timestamp,
                    line_type: if source_allegiance == target_allegiance {ESOLogsLineType::BuffFadedAlly} else {ESOLogsLineType::BuffFadedEnemy},
                    buff_event,
                    unit_instance_id: instance_ids,
                    source_allegiance,
                    target_allegiance,
                    source_cast_index: None,
                    source_shield: source.shield,
                    target_shield: target.shield,
                }
            ));
        } else if parts[2] == "UPDATED" {
            if let Some(character) = self.eso_logs_log.esosim_characters.get_mut(&target.unit_id) {
                character.add_buff(ability_id.clone(), stacks as u8);
            }
            if stacks > 1 {
                if source_allegiance == 32 {source_allegiance = 16}
                if target_allegiance == 32 {target_allegiance = 16}
                let line_type = {
                    if source == target {ESOLogsLineType::StacksUpdatedSelf} else if source_allegiance == target_allegiance {ESOLogsLineType::BuffStacksUpdatedAlly} else {
                        ESOLogsLineType::BuffStacksUpdatedEnemy
                    }
                };
                self.add_log_event(ESOLogsEvent::StackUpdate (
                    ESOLogsBuffStacks {
                        timestamp: self.last_known_timestamp,
                        line_type,
                        buff_event,
                        unit_instance_id: instance_ids,
                        source_allegiance,
                        target_allegiance,
                        stacks,
                    }
                ));
            }
        }
        self.process_target_stats(target.unit_id, target_allegiance)?;
        // let target_unit_index = self.unit_index(target.unit_id).ok_or_else(|| format!("target_unit_index {} is out of bounds", target.unit_id))?;
        // let target_name = &self.eso_logs_log.units[target_unit_index].name;
        // if let Some(character) = self.eso_logs_log.esosim_characters.get_mut(&target.unit_id) {
        //     let max_health = target.max_health;
        //     let max_magicka = target.max_magicka;
        //     let max_stamina = target.max_stamina;

        //     if max_health > 0 && max_magicka > 0 && max_stamina > 0 {
        //         // character.handle_event(esosim::engine::ExternalResourceSource {health: target.max_health, magicka: target.max_magicka, stamina: target.max_stamina});
        //         let calc_max_health = character.get_max_health();
        //         let calc_max_magicka = character.get_max_magicka();
        //         let calc_max_stamina = character.get_max_stamina();

        //         log::debug!("Stat diff for unit {}:", target_name);
        //         Self::fmt_stat("Health",   max_health,   calc_max_health);
        //         Self::fmt_stat("Magicka",  max_magicka,  calc_max_magicka);
        //         Self::fmt_stat("Stamina",  max_stamina,  calc_max_stamina);
        //     }
        // }
        Ok(())
    }

    // fn diff_to_ansi_color(diff: i32) -> String {
    //     const GREEN_ZONE: i32 = 50;
    //     const MAX_DIFF: i32 = 4000;

    //     let abs = diff.abs();

    //     let abs = abs.min(MAX_DIFF);

    //     if abs <= GREEN_ZONE {
    //         return "\x1b[38;2;0;255;0m".to_string();
    //     }

    //     let t = (abs - GREEN_ZONE) as f32 / (MAX_DIFF - GREEN_ZONE) as f32;

    //     let r = (255.0 * t) as u8;
    //     let g = (255.0 * (1.0 - t)) as u8;

    //     format!("\x1b[38;2;{};{};0m", r, g)
    // }

    // fn fmt_stat(name: &str, target: u32, calc: u32) {
    //     let diff = target as i32 - calc as i32;
    //     // if diff != 0 {return}
    //     let colour = Self::diff_to_ansi_color(diff);
    //     log::debug!(
    //         "{:>8}: target={:<6} calc={:<6} diff={}{}\x1b[0m",
    //         name, target, calc, colour, diff
    //     );
    //     // debug_assert!(diff >= 0 || target < 12000);
    // }

    fn handle_map_changed(&mut self, parts: &[String]) -> Result<(), String> {
        let zone_id = parts[2].parse().unwrap_or(0);
        let zone_name: Arc<str> = parts[3].to_string().trim_matches('"').into();
        let map_url = parts[4].trim_matches('"').to_lowercase().into();
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().map_err(|e| format!("Failed to parse timestamp: {e}"))?);
        self.add_log_event(ESOLogsEvent::MapInfo(ESOLogsMapInfo {
            timestamp,
            line_type: ESOLogsLineType::MapInfo,
            map_id: zone_id,
            map_name: zone_name,
            map_image_url: map_url,
        }));
        Ok(())
    }

    fn handle_zone_changed(&mut self, parts: &[String]) -> Result<(), String> {
        let zone_id: u16 = parts[2].parse().unwrap_or(0);
        let zone_name = parts[3].to_string().trim_matches('"').into();
        let difficulty: String = parts[4].trim_matches('"').into();
        let difficulty_int = match difficulty.as_str() {
            "NONE" => 0,
            "NORMAL" => 1,
            "VETERAN" => 2,
            _ => {
                log::warn!("Unknown zone difficulty: {difficulty}");
                0
            }
        };
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().map_err(|e| format!("Failed to parse timestamp: {e}"))?);
        self.add_log_event(ESOLogsEvent::ZoneInfo(ESOLogsZoneInfo {
            timestamp,
            line_type: ESOLogsLineType::ZoneInfo,
            zone_id,
            zone_name,
            zone_difficulty: difficulty_int,
        }));
        self.zone = Some(zone_id);
        Ok(())
    }

    fn handle_trial_end(&mut self, parts: &[String]) -> Result<(), String> {
        let id = parts[2].parse::<u32>().unwrap_or(0);
        let duration = parts[3].parse::<u64>().unwrap_or(0);
        let success = parse::is_true(&parts[4]);
        let final_score = parts[5].parse::<u32>().unwrap_or(0);
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().map_err(|e| format!("Failed to parse timestamp: {e}"))?);
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
        Ok(())
    }

    const HEALTH_RECOVERY_BUFF_ID: u32 = 61322;
    fn handle_health_recovery(&mut self, parts: &[String]) -> Result<(), String> {
        let source = parse::unit_state(parts, 3);
        let source_id = self.unit_index(source.unit_id).ok_or_else(|| format!("health_recovery source_index {} is out of bounds", source.unit_id))?;
        let buff_index = self.buff_index(Self::HEALTH_RECOVERY_BUFF_ID).expect("health_recovery_buff_index should always exist");
        let mut buff_event = ESOLogsBuffEvent {
            unique_index: 0,
            source_unit_index: source_id,
            target_unit_index: source_id,
            buff_index,
        };
        let unique_index = self.add_buff_event(buff_event);
        buff_event.unique_index = unique_index;
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().map_err(|e| format!("Failed to parse timestamp: {e}"))?);
        let health_recovery = ESOLogsHealthRecovery {
            timestamp,
            line_type: ESOLogsLineType::HotTick,
            buff_event,
            effective_regen: parts[2].parse::<u32>().map_err(|e| format!("Failed to parse effective_regen: {e}"))?,
            unit_state: ESOLogsUnitState { unit_state: source, champion_points: self.get_cp_for_unit(source.unit_id) }
        };
        self.add_log_event(ESOLogsEvent::HealthRecovery(health_recovery));
        Ok(())
    }

    fn handle_effect_info(&mut self, parts: &[String]) -> Result<(), String> {
        let effect_id: u32 = parts[2].parse().map_err(|e| format!("Failed to parse effect_id: {e}"))?;
        // let effect_type = effect::parse_effect_type(parts[3]);
        let status_effect_type = effect::parse_status_effect_type(&parts[4]);
        if let Some(&idx) = self.eso_logs_log.buffs_hashmap.get(&effect_id) {
            if let Some(buff) = self.eso_logs_log.buffs.get_mut(idx) {
                buff.status_type = status_effect_type;
            }
        }
        Ok(())
    }

    fn handle_unit_changed(&mut self, parts: &[String]) -> Result<(), String> {
        let unit_id = parts[2].parse().map_err(|e| format!("Failed to parse unit_id: {e}"))?;
        let unit_index = self.unit_index(unit_id);
        if unit_index.is_some() {
            let unit = &mut self.eso_logs_log.units[unit_index.ok_or_else(|| "Failed to unwrap unit_index".to_string())?];
            unit.unit_type = unit::match_reaction(&parts[11]);
        }
        Ok(())
    }

    fn handle_end_cast(&mut self, parts: &[String]) -> Result<(), String> {
        let end_reason = parse_cast_end_reason(&parts[2]);
        let timestamp = self.calculate_timestamp(parts[0].parse::<u64>().map_err(|e| format!("Failed to parse timestamp: {e}"))?);
        if end_reason == Some(CastEndReason::Interrupted) {
            let interrupted_cast_id = parts[3].parse::<u32>().map_err(|e| format!("Failed to parse interrupted_cast_id: {e}"))?;
            let interrupted_ability = parts[4].parse::<u32>().map_err(|e| format!("Failed to parse interrupted_ability_id: {e}"))?;
            let interrupting_ability = parts[5].parse::<u32>().map_err(|e| format!("Failed to parse interrupting_ability_id: {e}"))?;
            let interrupting_unit = parts[6].parse::<u32>().map_err(|e| format!("Failed to parse interrupting_unit_id: {e}"))?; // can be zero sometimes
            let mut target_id_option = self.eso_logs_log.cast_id_source_unit_id.get(&interrupted_cast_id).cloned();

            if interrupting_unit == 0 {return Err("Interrupting unit has id zero".to_string())}

            if target_id_option.is_none() {
                if let Some(last_interrupt) = &self.last_interrupt {
                    target_id_option = Some(*last_interrupt);
                } else {
                    return Err(format!("source for interrupted cast doesn't exist: {parts:?}"));
                }
            }

            let target_id = target_id_option.ok_or_else(|| "Failed to unwrap target_id_option".to_string())?;
            let target_index = self.eso_logs_log.unit_index(&target_id)
                .ok_or_else(|| format!("every target id should map to a unit: {target_id}"))?;
            let target = self.eso_logs_log.units.get(target_index)
                .ok_or_else(|| format!("every target index should be a unit: {target_index}"))?;
            let target_allegiance = Self::allegiance_from_reaction(target.unit_type);
            let target_session_index = self.eso_logs_log.index_in_session(&target_id).unwrap_or(0);
            let source_index = self.unit_index(interrupting_unit)
                .ok_or_else(|| format!("interrupting unit should always exist: {interrupting_unit}"))?;
            let source = self.eso_logs_log.units.get(source_index)
                .ok_or_else(|| format!("interrupting unit should always exist: {source_index}"))?;
            let source_allegiance = Self::allegiance_from_reaction(source.unit_type);
            let instance_id = self.eso_logs_log.index_in_session(&interrupting_unit).unwrap_or(0);
            let interrupted_ability_index = self.buff_index(interrupted_ability)
                .ok_or_else(|| format!("interrupted ability should always be something: {interrupted_ability}"))?;
            let interrupted_ability_from_table = self.eso_logs_log.buffs.get(interrupted_ability_index)
                .ok_or_else(|| format!("index should always be at a point into buffs: {interrupted_ability_index}"))?;
            let interrupting_ability_index = self.buff_index(interrupting_ability)
                .ok_or_else(|| format!("interrupting ability should be something: {interrupting_ability}"))?;
            let interrupting_ability_from_table = self.eso_logs_log.buffs.get(interrupting_ability_index)
                .ok_or_else(|| format!("index should always be at a point into buffs: {interrupting_ability_index}"))?;


            if interrupting_ability_from_table.name == interrupted_ability_from_table.name {return Err(format!("Ability {} interrupted itself", interrupted_ability_from_table.name))}

            let new_cast_key = ESOLogsBuffEventKey {
                source_unit_index: source_index,
                target_unit_index: target_index,
                buff_index: interrupting_ability_index,
            };

            let buff_index = self.eso_logs_log.effects_hashmap.get(&new_cast_key);
            if let Some(index) = buff_index {
                let buff = self.eso_logs_log.effects.get(*index).ok_or_else(|| format!("buff index {index} should always point to an effect"))?;

                self.add_log_event(ESOLogsEvent::Interrupt(
                ESOLogsInterrupt {
                    timestamp,
                    line_type: ESOLogsLineType::Interrupted,
                    buff_event: *buff,
                    unit_instance_id: (instance_id, target_session_index),
                    source_allegiance,
                    target_allegiance,
                    interrupted_ability_index,
                }));
            }
        } else if end_reason == Some(CastEndReason::Completed) {
            let ability_cast_id = parts[3].parse::<u32>()
                .map_err(|e| format!("Failed to parse ability_cast_id: {e}"))?;

            if !self.eso_logs_log.cast_with_cast_time.contains(&ability_cast_id) {
                return Ok(())
            }
            // log::trace!("Ability cast id: {}", ability_cast_id);

            let buff_index = self.eso_logs_log.cast_id_hashmap
                .get(&ability_cast_id)
                .unwrap_or(&usize::MAX);
            if *buff_index == usize::MAX {
                return Err("Completed cast buff index is none".to_string())
            }

            let buff = *self.eso_logs_log.effects
                .get(*buff_index)
                .ok_or_else(|| format!("buff_index {buff_index} is out of bounds in effects"))?;
            let caster_id_option = self.eso_logs_log.cast_id_source_unit_id.get(&ability_cast_id);
            if caster_id_option.is_none() {
                return Err(format!("caster_id of completed cast of ability cast id {ability_cast_id} is none"))
            }

            let caster_id = *caster_id_option
                .ok_or_else(|| "Failed to unwrap caster_id_option".to_string())?;
            let caster_index = self.eso_logs_log.unit_index(&caster_id)
                .ok_or_else(|| format!("every target id should map to a unit: {caster_id}"))?;
            let caster = self.eso_logs_log.units
                .get(caster_index)
                .ok_or_else(|| format!("every target index should be a unit: {caster_index}"))?;
            let caster_allegiance = Self::allegiance_from_reaction(caster.unit_type);
            let caster_session_index = self.eso_logs_log.index_in_session(&caster_id).unwrap_or(0);
            let target_id = *self.eso_logs_log.cast_id_target_unit_id
                .get(&ability_cast_id)
                .ok_or_else(|| format!("every cast id should have a target: {ability_cast_id}"))?;
            let (target_allegiance, target_session_index) = if target_id != 0 {
                let target_index = self.eso_logs_log.unit_index(&target_id)
                    .ok_or_else(|| format!("every target id should have an index: {target_id}"))?;
                let target = self.eso_logs_log.units
                    .get(target_index)
                    .ok_or_else(|| format!("every target index should point to a unit: {target_index}"))?;
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
                    buff_event: buff,
                    unit_instance_id: (caster_session_index, target_session_index),
                    source_allegiance: caster_allegiance,
                    target_allegiance
                }
            ));
        }
        Ok(())
    }
}

pub fn split_and_zip_log_by_fight<InputPath, OutputDir, F>(input_path: InputPath, output_dir: OutputDir, mut progress_callback: F, cancel_flag: &AtomicBool) -> Result<(), String> where InputPath: AsRef<Path>, OutputDir: AsRef<Path>, F: FnMut(u8) {
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
    let lines = BufReader::new(input_file).lines();

    let mut elp = ESOLogProcessor::new();
    let mut custom_state = CustomLogData::new();
    let mut fight_index: u16 = 1;

    let mut first_timestamp: Option<u64> = None;
    let mut current_line: usize = 0;
    for line in lines {
        current_line += 1;
        if current_line % LINE_COUNT_FOR_PROGRESS == 0 {
            progress_callback(((current_line as f64 / total_lines as f64) * 100.0).round() as u8);
            if cancel_flag.load(Ordering::SeqCst) {
                return Err("Upload cancelled".to_string());
            }
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
            elp.remove_overabundant_events();
            let seg_zip = output_dir
                .as_ref()
                .join(format!("report_segment_{fight_index}.zip"));
            let seg_data = build_report_segment(&elp);
            write_zip_with_logtxt(seg_zip, seg_data.as_bytes())?;

            let events = &elp.eso_logs_log.events;
            if !events.is_empty() {
                let mut last_ts = event_timestamp(&events[events.len()-1]);
                if last_ts.is_some() && first_timestamp.is_some() {
                    last_ts = Some(last_ts.ok_or_else(|| "Failed to unwrap last_timestamp".to_string())? + first_timestamp.ok_or_else(|| "Failed to unwrap first timestamp".to_string())?);
                }
                if let (Some(first), Some(last)) = (first_timestamp, last_ts) {
                    use std::io::Write;
                    let timestamps_path = output_dir.as_ref().join("timestamps");
                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(timestamps_path)
                        .map_err(|e| format!("Failed to open timestamps file: {e}"))?;
                    writeln!(file, "{first},{last}")
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
        .join("master_table.zip");
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
    let server_id = if elp.megaserver == "NA Megaserver".into() { 1 } else { 2 };

    out.push_str(&format!("15|{server_id}\n"));
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

    let default_icon = "ability_mage_065".into();
    let mut icon_by_name = std::collections::HashMap::<Arc<str>, Arc<str>>::new();
    for buff in elp.eso_logs_log.buffs.iter_mut() {
        let new_icon = match buff.id {
            135924 => Some("gear_seagiant_staff".into()), // RO cooldown
            193447 => Some("u38_antiquities_goldandblueshalknecklace".into()), // velothi
            189533 => Some("ability_arcanist_002".into()), // fatecarver
            188456 => Some("gear_undinfernium_head_a".into()), // ozezan
            154820 => Some("gear_rockgrove_heavy_head_a".into()), // saxhleel
            157738 => Some("gear_rockgrove_med_head_a".into()), // sul-xan
            111504 => Some("gear_undaunted_werewolfbehemoth_head_a".into()), // balorgh
            220015 => Some("gear_lucentguardian_heavy_head_a".into()), // lucent echoes
            147459 => Some("antiquities_ornate_necklace_3".into()), // pearls of ehlnofey
            117714 | 117693 => Some("ability_necromancer_002_a".into()), // blastbones grey-ed out
            89109 => Some("ability_warden_017_b".into()), // Bull Netch (thanks sparkrip)
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
            }
        }

        if matches!(buff.id,
            86304 // lifesteal
            | 172672 // whorl of the depths
            | 156020 // from the brink
            | 190960 // harmony (jewellery synergy)
            | 103966 // concentrated barrier
            | 160827 // selene
            | 133494 // aegis caller
            | 220863 // sliver assault
        ) {
            buff.caused_by_id = 0;
        }

        if buff.id == buff.caused_by_id && buff.damage_type == DamageType::None {
            buff.caused_by_id = 0;
        }

        if matches!(buff.caused_by_id,
            26770
        ) {
            buff.caused_by_id = 0;
        }

        buff.damage_type = match buff.id {
            103631 | 103622 => DamageType::Oblivion, // roaring flare
            _ => buff.damage_type,
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
                    elp.eso_logs_log.buffs[i].caused_by_id = 0;
                }
            }
        }
    }

    let server_id = if elp.megaserver == "\"NA Megaserver\"".into() { 1 } else { 2 };
    out.push_str(&format!("15|{server_id}|\n"));

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