use std::collections::HashMap;

use crate::{effect::{self, Ability, EffectChangeType}, event::{self, CastEndReason}, fight::Fight, parse, player::Player, unit::{Unit, UnitType}};

pub struct Log {
    pub log_epoch: i64,
    pub players: HashMap<u32, crate::player::Player>,
    pub units: HashMap<u32, crate::unit::Unit>,
    pub fights: Vec<crate::fight::Fight>,
    pub abilities: HashMap<u32, crate::effect::Ability>,
    pub effects: HashMap<u32, crate::effect::Effect>,
}

impl Log {
    pub fn new() -> Self {
        let new_self: Self = Self {
            log_epoch: 0,
            players: HashMap::new(),
            units: HashMap::new(),
            fights: Vec::new(),
            abilities: HashMap::new(),
            effects: HashMap::new(),
        };

        new_self
    }

    pub fn parse_line(&mut self, parts: Vec<&str>) {    
        match parts[1] {
            "BEGIN_LOG" => self.handle_begin_log(parts),
            "BEGIN_COMBAT" | "END_COMBAT" => self.handle_combat_change(parts),
            "UNIT_ADDED" => self.handle_unit_added(&parts),
            "PLAYER_INFO" => self.handle_player_info(&parts),
            "ABILITY_INFO" => self.handle_ability_info(&parts),
            "EFFECT_INFO" => self.handle_effect_info(&parts),
            "COMBAT_EVENT" => self.handle_combat_event(&parts),
            "BEGIN_CAST" => self.handle_begin_cast(&parts),
            "END_CAST" => self.handle_end_cast(parts),
            "HEALTH_REGEN" => {},
            "UNIT_CHANGED" => {},
            "UNIT_REMOVED" => {},
            "EFFECT_CHANGED" => self.handle_effect_changed(&parts),
            "MAP_CHANGED" => {},
            "ZONE_CHANGED" => {},
            "TRIAL_INIT" => {},
            "BEGIN_TRIAL" => {},
            "END_TRIAL" => {},
            "ENDLESS_DUNGEON_BEGIN" | "ENDLESS_DUNGEON_STAGE_END" | "ENDLESS_DUNGEON_BUFF_ADDED" | "ENDLESS_DUNGEON_BUFF_REMOVED" | "ENDLESS_DUNGEON_END" => {},
            _ => println!("{}{}", "Unknown log line type: ", parts[1]),
        }
    }

    pub fn get_player_by_id(&mut self, id: u32) -> Option<&mut Player> {
        self.players.get_mut(&id)
    }

    pub fn get_unit_by_id(&mut self, id: u32) -> Option<&mut Unit> {
        self.units.get_mut(&id)
    }

    pub fn get_ability_by_id(&mut self, id: u32) -> Option<&mut Ability> {
        self.abilities.get_mut(&id)
    }

    pub fn get_readonly_unit_by_id(&self, id: u32) -> Option<&Unit> {
        self.units.get(&id)
    }

    fn get_current_fight_readonly(&self) -> Option<&Fight> {
        self.fights.last().filter(|fight| fight.end_time == 0)
    }

    fn get_current_fight(&mut self) -> Option<&mut Fight> {
        self.fights.last_mut().filter(|fight| fight.end_time == 0)
    }

    fn handle_begin_log(&mut self, parts: Vec<&str>) {
        self.log_epoch = parts[2].parse::<i64>().unwrap();
        let log_version = parts[3];
        if log_version != "15" {
            println!("Unknown log version: {} (Expected 15)", log_version);
        }
    }

    fn handle_unit_added(&mut self, parts: &[&str]) {
        match parts[3] {
            "PLAYER" => {
                let player = parse::player(parts);
                if !player.display_name.is_empty() {
                    self.players.insert(player.unit_id, player);
                }
            }
            "MONSTER" => {
                let monster = parse::monster(parts);
                self.units.insert(monster.unit_id, monster);
            }
            "OBJECT" => {
                let object = parse::object(parts);
                self.units.insert(object.unit_id, object);
            }
            _ => {}
        }
    }
    
    fn handle_combat_change(&mut self, parts: Vec<&str>) {
        if parts[1] == "BEGIN_COMBAT" {
            let mut new_fight = crate::fight::Fight {
                id: self.fights.len() as u16,
                name: "Unknown".to_string(),
                players: Vec::new(),
                monsters: Vec::new(),
                start_time: parts[0].parse::<u64>().unwrap(),
                end_time: 0,
                events: Vec::new(),
                casts: Vec::new(),
                effect_events: Vec::new(),
            };
            for player in self.players.values() {
                new_fight.players.push(player.clone());
            }
            self.fights.push(new_fight);
        } else if parts[1] == "END_COMBAT" {
            if let Some(fight) = self.get_current_fight_readonly() {
                let mut candidate_ids = Vec::new();
                let mut max_hp_seen: u32 = 0;
                let mut max_hp_id: u32 = 0;
                for event in &fight.events {
                    let unit_id = event.target_unit_state.unit_id;
                    if !candidate_ids.contains(&unit_id) {
                        candidate_ids.push(unit_id);
                    }
                    if event.target_unit_state.max_health > max_hp_seen {
                        max_hp_seen = event.target_unit_state.max_health;
                        max_hp_id = unit_id;
                    }
                }

                let candidate_ids_cloned = candidate_ids.clone();
                let max_hp_id_cloned = max_hp_id;


                let candidate_units: Vec<_> = candidate_ids_cloned.iter()
                    .filter_map(|id| self.units.get(id))
                    .collect();
                let boss_name = candidate_units.iter()
                    .find(|unit| unit.is_boss)
                    .map(|boss| boss.name.to_string());
                let unit_name = self
                    .get_readonly_unit_by_id(max_hp_id_cloned)
                    .map(|unit| unit.name.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                if let Some(fight) = self.get_current_fight() {
                    fight.name = if let Some(name) = boss_name {
                        name
                    } else {
                        unit_name
                    }
                }
            }
            if let Some(fight) = self.get_current_fight() {
                fight.end_time = parts[0].parse::<u64>().unwrap();
            }
        }
    }

    // 3597,PLAYER_INFO,1,[142079,78219,72824,150054,147459,46751,39248,35770,46041,33090,70390,117848,45301,63802,13984,34741,61930,135397,203342,215493,122586,120017,61685,120023,61662,120028,61691,120029,61666,120008,61744,120015,109966,177885,147417,93109,120020,88490,120021,120025,120013,61747,177886,120024,120026],[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,0,1,1,2,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],[[HEAD,185032,T,8,ARMOR_PROSPEROUS,ARCANE,640,INVALID,F,0,NORMAL],[NECK,171437,T,16,JEWELRY_ARCANE,LEGENDARY,576,INCREASE_BASH_DAMAGE,F,35,ARCANE],[CHEST,45095,T,16,ARMOR_REINFORCED,LEGENDARY,0,PRISMATIC_DEFENSE,F,5,LEGENDARY],[SHOULDERS,56058,F,12,ARMOR_NIRNHONED,MAGIC,0,INVALID,F,0,NORMAL],[OFF_HAND,184873,T,6,ARMOR_DIVINES,ARCANE,640,INVALID,F,0,NORMAL],[WAIST,184888,F,1,ARMOR_DIVINES,NORMAL,640,INVALID,F,0,NORMAL],[LEGS,45169,T,1,ARMOR_TRAINING,ARCANE,0,MAGICKA,F,35,MAGIC],[FEET,45061,F,50,ARMOR_IMPENETRABLE,ARTIFACT,0,MAGICKA,F,35,ARCANE],[COSTUME,55262,F,1,NONE,ARCANE,0,INVALID,F,0,NORMAL],[RING1,139657,F,1,JEWELRY_BLOODTHIRSTY,ARTIFACT,0,INVALID,F,0,NORMAL],[RING2,44904,F,0,NONE,LEGENDARY,0,INVALID,F,0,NORMAL],[BACKUP_POISON,79690,F,1,NONE,LEGENDARY,0,INVALID,F,0,NORMAL],[HAND,185058,F,28,ARMOR_STURDY,NORMAL,640,HEALTH,F,30,NORMAL],[BACKUP_MAIN,185007,T,12,WEAPON_CHARGED,MAGIC,640,DAMAGE_SHIELD,F,35,ARTIFACT],[BACKUP_OFF,184897,T,12,WEAPON_PRECISE,NORMAL,640,FROZEN_WEAPON,F,35,ARCANE]],[25267,61919,34843,36901,25380,113105],[36935,35419,61507,34727,36028]
    fn handle_player_info(&mut self, parts: &[&str]) {
        let unit_id: u32 = parts[2].parse().unwrap();
        let primary_ids: Vec<u32> = parts[parts.len() - 2]
            .split(',')
            .filter_map(|x| x.parse().ok())
            .collect();
        let backup_ids: Vec<u32> = parts[parts.len() - 1]
            .split(',')
            .filter_map(|x| x.parse().ok())
            .collect();

        let (primary, backup): (Vec<_>, Vec<_>) = {
            let get = |id: &u32, p: &Player| {
                p.primary_abilities
                    .iter()
                    .find(|a| a.id == *id)
                    .cloned()
                    .or_else(|| self.abilities.get(id).cloned())
            };
            if let Some(global_p) = self.players.get(&unit_id) {
                (
                    primary_ids
                        .iter()
                        .filter_map(|id| get(id, global_p))
                        .collect(),
                    backup_ids
                        .iter()
                        .filter_map(|id| get(id, global_p))
                        .collect(),
                )
            } else {
                (
                    primary_ids
                        .iter()
                        .filter_map(|id| self.abilities.get(id).cloned())
                        .collect(),
                    backup_ids
                        .iter()
                        .filter_map(|id| self.abilities.get(id).cloned())
                        .collect(),
                )
            }
        };

        if let Some(fight) = self.get_current_fight() {
            let effect_ids: Vec<u32> = parts[3]
                .split(',')
                .filter_map(|x| x.parse().ok())
                .collect();
            if let Some(player) = fight.players.iter_mut().find(|p| p.unit_id == unit_id) {
                for eid in effect_ids {
                    if !player.effects.contains(&eid) {
                        player.effects.push(eid);
                    }
                }
                // gear
                for gear_idx in 5..(parts.len() - 2) {
                    let gear_piece = parse::gear_piece(parts[gear_idx]);
                    player.insert_gear_piece(gear_piece);
                }
                player.primary_abilities = primary;
                player.backup_abilities = backup;
                effect::destruction_staff_skill_convert(player);
            }
        }
    }

    fn handle_ability_info(&mut self, parts: &[&str]) {
        let ability = parse::ability(parts);
        self.abilities.insert(ability.id, ability);
    }

    //360508,ABILITY_INFO,26874,"Blazing Spear","/esoui/art/icons/ability_templar_sun_strike.dds",F,T
    //360508,EFFECT_INFO,26874,BUFF,NONE,NEVER,26832
    fn handle_effect_info(&mut self, parts: &[&str]) {
        let effect = parse::effect(parts, &self.abilities);
        self.effects.insert(effect.ability.id, effect);
    }

    fn handle_combat_event(&mut self, parts: &[&str]) {
        let source = parse::unit_state(parts, 9);
        let target = if parts[19] == "*" {
            source.clone()
        } else {
            parse::unit_state(parts, 19)
        };

        if event::parse_event_result(parts[2]).is_none() {
            println!("Unknown event result: {}", parts[2]);
        }
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

        let monsters: Vec<_> = [&ev.source_unit_state, &ev.target_unit_state]
            .iter()
            .filter_map(|s| self.units.get(&s.unit_id))
            .filter(|u| u.unit_type == UnitType::Monster)
            .cloned()
            .collect();

        if let Some(fight) = self.get_current_fight() {
            fight.events.push(ev);
            for m in monsters {
                if !fight.monsters.iter().any(|x| x.unit_id == m.unit_id) {
                    fight.monsters.push(m);
                }
            }
        }
    }

    fn handle_begin_cast(&mut self, parts: &[&str]) {
        let source = parse::unit_state(parts, 6);
        let target = if parts[16] == "*" {
            source.clone()
        } else {
            parse::unit_state(parts, 16)
        };
        let cast = event::Cast {
            time: parts[0].parse().unwrap(),
            duration: parts[2].parse().unwrap(),
            channeled: parts[3] == "T",
            cast_track_id: parts[4].parse().unwrap(),
            ability_id: parts[5].parse().unwrap(),
            source_unit_state: source,
            target_unit_state: target,
            interrupt_reason: None,
        };
        if let Some(fight) = self.get_current_fight() {
            fight.casts.push(cast);
        }
    }

    fn handle_end_cast(&mut self, parts: Vec<&str>) {
        if crate::event::parse_cast_end_reason(parts[2]) != Some(CastEndReason::Completed) {
            let time = parts[0].parse::<u64>().unwrap();
            let cast_id = parts[3].parse::<u32>().unwrap();
            if let Some(fight) = self.get_current_fight() {
                if let Some(cast) = fight.casts.iter_mut().rev().find(|cast| cast.cast_track_id == cast_id) {
                    cast.duration = (time - cast.time) as u32;
                }
            }
        }
    }

    fn handle_effect_changed(&mut self, parts: &[&str]) {
        let source = parse::unit_state(parts, 6);
        let target = if parts[16] == "*" {
            source.clone()
        } else {
            parse::unit_state(parts, 16)
        };
        let ev = effect::EffectEvent {
            time: parts[0].parse().unwrap(),
            change_type: effect::parse_effect_change_type(parts[2]),
            stack_count: parts[3].parse().unwrap(),
            cast_track_id: parts[4].parse().unwrap(),
            ability_id: parts[5].parse().unwrap(),
            source_unit_state: source,
            target_unit_state: target,
            player_initiated_remove_cast_track_id: false,
        };

        let unit_id = target.unit_id;
        let effect_id = ev.ability_id;
        match (
            self.players.get_mut(&unit_id),
            self.units.get_mut(&unit_id),
            ev.change_type,
        ) {
            (Some(player), _, EffectChangeType::Gained) if !player.effects.contains(&effect_id) => {
                player.effects.push(effect_id)
            }
            (Some(player), _, EffectChangeType::Faded) => {
                player.effects.retain(|&id| id != effect_id)
            }
            (None, Some(mon), EffectChangeType::Gained) if !mon.effects.contains(&effect_id) => {
                mon.effects.push(effect_id)
            }
            (None, Some(mon), EffectChangeType::Faded) => {
                mon.effects.retain(|&id| id != effect_id)
            }
            _ => {}
        }

        if let Some(fight) = self.get_current_fight() {
            fight.effect_events.push(ev);
        }
    }
}