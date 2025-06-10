use std::collections::HashMap;

use crate::{effect::EffectChangeType, event::CastEndReason};

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
            "UNIT_ADDED" => self.handle_unit_added(parts),
            "PLAYER_INFO" => self.handle_player_info(parts),
            "ABILITY_INFO" => self.handle_ability_info(parts),
            "EFFECT_INFO" => self.handle_effect_info(parts),
            "COMBAT_EVENT" => self.handle_combat_event(parts),
            "BEGIN_CAST" => self.handle_begin_cast(parts),
            "END_CAST" => self.handle_end_cast(parts),
            "HEALTH_REGEN" => {},
            "UNIT_CHANGED" => {},
            "UNIT_REMOVED" => {},
            "EFFECT_CHANGED" => self.handle_effect_changed(parts),
            "MAP_CHANGED" => {},
            "ZONE_CHANGED" => {},
            "TRIAL_INIT" => {},
            "BEGIN_TRIAL" => {},
            "END_TRIAL" => {},
            "ENDLESS_DUNGEON_BEGIN" | "ENDLESS_DUNGEON_STAGE_END" | "ENDLESS_DUNGEON_BUFF_ADDED" | "ENDLESS_DUNGEON_BUFF_REMOVED" | "ENDLESS_DUNGEON_END" => {},
            _ => println!("{}{}", "Unknown log line type: ", parts[1]),
        }
    }

    pub fn get_player_by_id(&mut self, id: u32) -> Option<&mut crate::player::Player> {
        self.players.get_mut(&id)
    }

    pub fn get_unit_by_id(&mut self, id: u32) -> Option<&mut crate::unit::Unit> {
        self.units.get_mut(&id)
    }

    pub fn get_ability_by_id(&mut self, id: u32) -> Option<&mut crate::effect::Ability> {
        self.abilities.get_mut(&id)
    }

    pub fn get_readonly_unit_by_id(&self, id: u32) -> Option<&crate::unit::Unit> {
        self.units.get(&id)
    }

    pub fn is_empty(&mut self) -> bool {
        self.fights.is_empty()
    }
    
    fn get_current_fight_readonly(&self) -> Option<&crate::fight::Fight> {
        self.fights.last().filter(|fight| fight.end_time == 0)
    }

    fn get_current_fight(&mut self) -> Option<&mut crate::fight::Fight> {
        self.fights.last_mut().filter(|fight| fight.end_time == 0)
    }

    fn is_true(value: &str) -> bool {
        value == "T"
    }

    fn handle_begin_log(&mut self, parts: Vec<&str>) {
        self.log_epoch = parts[2].parse::<i64>().unwrap();
        let log_version = parts[3];
        if log_version != "15" {
            println!("Unknown log version: {} (Expected 15)", log_version);
        }
    }

    fn handle_unit_added(&mut self, parts: Vec<&str>) {
        let unit_id: u32 = parts[2].parse::<u32>().unwrap();
        if parts[3] == "PLAYER" {
            let display_name = parts[11].trim_matches('"').to_string();
            let name = parts[10].to_string();
            let player_per_session_id: u32 = parts[5].parse::<u32>().unwrap();
            if display_name != "" {
                let player = crate::player::Player {
                    unit_id: unit_id,
                    is_local_player: Self::is_true(parts[4]),
                    player_per_session_id: player_per_session_id,
                    class_id: crate::player::match_class(parts[8]),
                    race_id: crate::player::match_race(parts[9]),
                    name: name,
                    display_name: display_name,
                    character_id: parts[12].parse::<u64>().unwrap(),
                    level: parts[13].parse::<u8>().unwrap(),
                    champion_points: parts[14].parse::<u16>().unwrap(),
                    is_grouped_with_local_player: Self::is_true(parts[17]),
                    unit_state: crate::unit::blank_unit_state(),
                    effects: Vec::new(),
                    gear: crate::player::empty_loadout(),
                    primary_abilities: Vec::new(),
                    backup_abilities: Vec::new(),
                };
                self.players.insert(unit_id, player);
            }
        } else if parts[3] == "MONSTER" {
            let name = parts[10].trim_matches('"').to_string();
            let monster = crate::unit::Unit {
                unit_id: unit_id,
                unit_type: crate::unit::UnitType::Monster,
                monster_id: parts[6].parse::<u32>().unwrap(),
                is_boss: Self::is_true(parts[7]),
                name: name,
                level: parts[13].parse::<u8>().unwrap(),
                champion_points: parts[14].parse::<u16>().unwrap(),
                owner_unit_id: parts[15].parse::<u32>().unwrap(),
                reaction: crate::unit::match_reaction(parts[16]),
                unit_state: crate::unit::blank_unit_state(),
                effects: Vec::new(),
            };
            self.units.insert(unit_id, monster);
        } else if parts[3] == "OBJECT" {
            let object = crate::unit::Unit {
                unit_id: unit_id,
                unit_type: crate::unit::UnitType::Object,
                monster_id: 0,
                is_boss: false,
                name: parts[10].to_string(),
                level: parts[13].parse::<u8>().unwrap(),
                champion_points: parts[14].parse::<u16>().unwrap(),
                owner_unit_id: parts[15].parse::<u32>().unwrap(),
                reaction: crate::unit::match_reaction(parts[16]),
                unit_state: crate::unit::blank_unit_state(),
                effects: Vec::new(),
            };
            self.units.insert(unit_id, object);
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
                // Move candidate_ids and max_hp_id out of the mutable borrow scope
                let candidate_ids_cloned = candidate_ids.clone();
                let max_hp_id_cloned = max_hp_id;

                // Now borrow self.units
                let candidate_units: Vec<_> = candidate_ids_cloned.iter()
                    .filter_map(|id| self.units.get(id))
                    .collect();
                let boss_name = candidate_units.iter()
                    .find(|unit| unit.is_boss)
                    .map(|boss| boss.name.to_string());
                let unit_name = self
                    .get_readonly_unit_by_id(max_hp_id_cloned)
                    .map(|unit| unit.name.to_string())
                    .unwrap_or_else(|| "Default".to_string());
                // Re-borrow fight as mutable to set the name
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
    fn handle_player_info(&mut self, parts: Vec<&str>) {
        let unit_id: u32 = parts[2].parse::<u32>().unwrap();
        let primary_ability_id_list: Vec<u32> = parts[parts.len() - 2].split(',').map(|x| x.parse::<u32>().unwrap_or_default()).collect();
        let backup_ability_id_list: Vec<u32> = parts[parts.len() - 1].split(',').map(|x| x.parse::<u32>().unwrap_or_default()).collect();
        let primary_abilities_to_add: Vec<_> = primary_ability_id_list.iter().filter_map(|id| self.abilities.get(id).cloned()).collect();
        let backup_abilities_to_add: Vec<_> = backup_ability_id_list.iter().filter_map(|id| self.abilities.get(id).cloned()).collect();

        if let Some(player) = self.get_player_by_id(unit_id) {
            for ability in primary_abilities_to_add {
                if !player.primary_abilities.iter().any(|a| a.id == ability.id) {
                    player.primary_abilities.push(ability);
                }
            }
            for ability in backup_abilities_to_add {
                if !player.primary_abilities.iter().any(|a| a.id == ability.id) {
                    player.primary_abilities.push(ability);
                }
            }
        }

        let get_ability_from_player = |id: &u32, player: &crate::player::Player| {
            player.primary_abilities.iter()
                .find(|a| a.id == *id)
                .cloned()
                .or_else(|| self.abilities.get(id).cloned())
        };

        // Get ability from global player to keep scribing scripts
        let (primary_abilities_to_add, backup_abilities_to_add) = if let Some(global_player) = self.players.get(&unit_id) {(
                primary_ability_id_list.iter().filter_map(|id| get_ability_from_player(id, global_player)).collect::<Vec<_>>(),
                backup_ability_id_list.iter().filter_map(|id| get_ability_from_player(id, global_player)).collect::<Vec<_>>()
            )} else {( // else grab it from global ability table
                primary_ability_id_list.iter().filter_map(|id| self.abilities.get(id).cloned()).collect::<Vec<_>>(),
                backup_ability_id_list.iter().filter_map(|id| self.abilities.get(id).cloned()).collect::<Vec<_>>()
            )
        };

        if let Some(fight) = self.get_current_fight() {
            let effect_id_list: Vec<u32> = parts[3].split(',').map(|x| x.parse::<u32>().unwrap()).collect();
            if let Some(player) = fight.players.iter_mut().find(|p| p.unit_id == unit_id) {
                // Add effects if not already present
                for effect_id in &effect_id_list {
                    if !player.effects.contains(effect_id) {
                        player.effects.push(*effect_id);
                    }
                }

                let gear_parts = parts.len() - 2;
                for i in 5..gear_parts {
                    let gear_piece = Self::handle_equipment_info(parts[i]);
                    player.insert_gear_piece(gear_piece);
                }

                player.primary_abilities.clear();
                player.backup_abilities.clear();

                for ability in primary_abilities_to_add {
                    player.primary_abilities.push(ability);
                }
                for ability in backup_abilities_to_add {
                    player.backup_abilities.push(ability);
                }
                crate::effect::destruction_staff_skill_convert(player);
            }
        }
    }

    fn handle_equipment_info(part: &str) -> crate::player::GearPiece {
        let split: Vec<&str> = part.split(",").collect();
        // check all enums for none values, and print what they are
        if crate::player::match_gear_slot(split[0]) == crate::player::GearSlot::None {
            println!("Unknown gear slot: {}", split[0]);
            // println!("{}", part);
        }
        if crate::player::match_gear_trait(split[4]) == crate::player::GearTrait::None && split[4] != "NONE" {
            println!("Unknown gear trait: {}", split[4]);
            // println!("{}", part);
        }
        if crate::player::match_gear_quality(split[5]) == crate::player::GearQuality::None {
            println!("Unknown gear quality: {}", split[5]);
            // println!("{}", part);
        }
        if crate::player::match_enchant_type(split[7]) == crate::player::EnchantType::None {
            println!("Unknown enchant type: {}", split[7]);
            // println!("{}", part);
        }
        let gear_piece = crate::player::GearPiece {
            slot: crate::player::match_gear_slot(split[0]),
            item_id: split[1].parse::<u32>().unwrap(),
            is_cp: Self::is_true(split[2]),
            level: split[3].parse::<u8>().unwrap(),
            gear_trait: crate::player::match_gear_trait(split[4]),
            quality: crate::player::match_gear_quality(split[5]),
            set_id: split[6].parse::<u16>().unwrap(),
            enchant: crate::player::GearEnchant {
                enchant_type: crate::player::match_enchant_type(split[7]),
                is_cp: Self::is_true(split[8]),
                enchant_level: split[9].parse::<u8>().unwrap(),
                enchant_quality: crate::player::match_gear_quality(split[10]),
            },
        };
        gear_piece
    }

    fn handle_ability_info(&mut self, parts: Vec<&str>) {
        let effect_id: u32 = parts[2].parse::<u32>().unwrap(); // abilityId usually unique, but can be reused for scribing abilities
        let name = parts[3].trim_matches('"').to_string();
        let ability = crate::effect::Ability {
                id: effect_id,
                name: name,
                icon: parts[4].trim_matches('"').split('/').last().unwrap().replace(".dds", ".png").to_string(),
                interruptible: Self::is_true(parts[5]),
                blockable: Self::is_true(parts[6]),
                scribing: if parts.len() == 10 {
                    let mut scribing = Vec::new();
                    for i in 7..10 {
                        scribing.push(parts[i].trim_matches('"').to_owned())
                    }
                    Some(scribing)
                } else {
                    None
                },
            };
        self.abilities.insert(effect_id, ability);
    }

    //360508,ABILITY_INFO,26874,"Blazing Spear","/esoui/art/icons/ability_templar_sun_strike.dds",F,T
    //360508,EFFECT_INFO,26874,BUFF,NONE,NEVER,26832
    fn handle_effect_info(&mut self, parts: Vec<&str>) {
        let effect_id: u32 = parts[2].parse::<u32>().unwrap();
        let ability = self.abilities.get(&effect_id).unwrap().clone();
        let effect = crate::effect::Effect {
            ability: ability,
            stack_count: 0,
            effect_type: crate::effect::parse_effect_type(parts[3]),
            status_effect_type: crate::effect::parse_status_effect_type(parts[4]),
            synergy: if parts.len() > 6 {parts[6].parse::<u32>().ok()} else {None},
        };
        self.effects.insert(effect_id, effect);
    }
    
    fn parse_unit_state(&mut self, parts: Vec<&str>, start_index: usize) -> crate::unit::UnitState {
        let parse_value = |s: &str| s.parse::<u32>().unwrap_or(0);
        let parse_pair = |s: &str| {
            let mut split = s.split('/');
            (
                split.next().map_or(0, parse_value),
                split.next().map_or(0, parse_value),
            )
        };

        crate::unit::UnitState {
            unit_id: parts[start_index].parse::<u32>().unwrap_or(0),
            health: parse_pair(parts[start_index + 1]).0,
            max_health: parse_pair(parts[start_index + 1]).1,
            magicka: parse_pair(parts[start_index + 2]).0,
            max_magicka: parse_pair(parts[start_index + 2]).1,
            stamina: parse_pair(parts[start_index + 3]).0,
            max_stamina: parse_pair(parts[start_index + 3]).1,
            ultimate: parse_pair(parts[start_index + 4]).0,
            max_ultimate: parse_pair(parts[start_index + 4]).1,
            werewolf: parse_pair(parts[start_index + 5]).0,
            werewolf_max: parse_pair(parts[start_index + 5]).1,
            shield: parts[start_index + 6].parse::<u32>().unwrap_or(0),
            map_x: parts[start_index + 7].parse::<f32>().unwrap_or(0.0),
            map_y: parts[start_index + 8].parse::<f32>().unwrap_or(0.0),
            heading: parts[start_index + 9].parse::<f32>().unwrap_or(0.0),
        }
    }

    fn handle_combat_event(&mut self, parts: Vec<&str>) {
        let source_unit_state = self.parse_unit_state(parts.clone(), 9);
        let target_unit_state = if parts[19] == "*" {
            source_unit_state.clone()
        } else {
            self.parse_unit_state(parts.clone(), 19)
        };

        if crate::event::parse_event_result(parts[2]).is_none() {
            println!("Unknown event result: {}", parts[2]);
        }
        let event = crate::event::Event {
            time: parts[0].parse::<u64>().unwrap(),
            result: crate::event::parse_event_result(parts[2]).unwrap(),
            damage_type: crate::event::parse_damage_type(parts[3]),
            power_type: parts[4].parse::<u32>().unwrap(),
            hit_value: parts[5].parse::<u32>().unwrap(),
            overflow: parts[6].parse::<u32>().unwrap(),
            cast_track_id: parts[7].parse::<u32>().unwrap(),
            ability_id: parts[8].parse::<u32>().unwrap(),
            source_unit_state: source_unit_state,
            target_unit_state: target_unit_state,
        };

        let mut monster_units = Vec::new();
        for unit_state in [&source_unit_state, &target_unit_state] {
            if let Some(unit) = self.units.get(&unit_state.unit_id) {
                if unit.unit_type == crate::unit::UnitType::Monster {
                    monster_units.push(unit.clone());
                }
            }
        }

        if let Some(fight) = self.get_current_fight() {
            fight.events.push(event);
            for unit in monster_units {
                if !fight.monsters.iter().any(|m| m.unit_id == unit.unit_id) {
                    fight.monsters.push(unit);
                }
            }
        }
    }

    fn handle_begin_cast(&mut self, parts: Vec<&str>) {
        let source_unit_state = self.parse_unit_state(parts.clone(), 6);
        let target_unit_state = if parts[16] == "*" {
            source_unit_state.clone()
        } else {
            self.parse_unit_state(parts.clone(), 16)
        };

        let cast = crate::event::Cast {
            time: parts[0].parse::<u64>().unwrap(),
            duration: parts[2].parse::<u32>().unwrap(),
            channeled: Self::is_true(parts[3]),
            cast_track_id: parts[4].parse::<u32>().unwrap(),
            ability_id: parts[5].parse::<u32>().unwrap(),
            source_unit_state: source_unit_state,
            target_unit_state: target_unit_state,
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

    fn handle_effect_changed(&mut self, parts: Vec<&str>) {
        let source_unit_state = self.parse_unit_state(parts.clone(), 6);
        let target_unit_state = if parts[16] == "*" {
            source_unit_state.clone()
        } else {
            self.parse_unit_state(parts.clone(), 16)
        };

        let effect_event = crate::effect::EffectEvent {
            time: parts[0].parse::<u64>().unwrap(),
            change_type: crate::effect::parse_effect_change_type(parts[2]),
            stack_count: parts[3].parse::<u16>().unwrap(),
            cast_track_id: parts[4].parse::<u32>().unwrap(),
            ability_id: parts[5].parse::<u32>().unwrap(),
            source_unit_state: source_unit_state,
            target_unit_state: target_unit_state,
            player_initiated_remove_cast_track_id: false, // what is an example where this is true?
        };

        let unit_id = target_unit_state.unit_id;
        let effect_id = effect_event.ability_id;

        // push effects onto/off unit object so that when player/monster is instantiated in next fight, the buffs are tracked appropriately
        if let Some(player) = self.players.get_mut(&unit_id) {
            if !player.effects.contains(&effect_id) && crate::effect::parse_effect_change_type(parts[2]) == EffectChangeType::Gained {
                player.effects.push(effect_id);
            } else if player.effects.contains(&effect_id) && crate::effect::parse_effect_change_type(parts[2]) == EffectChangeType::Faded {
                player.effects.retain(|&id| id != effect_id);
            }
            // ensures that buffs given to monsters before combat (instead of only players) such as via ele drain or vibrant shroud are known
        } else if let Some(monster) = self.units.get_mut(&unit_id) { 
            if !monster.effects.contains(&effect_id) && crate::effect::parse_effect_change_type(parts[2]) == EffectChangeType::Gained {
                monster.effects.push(effect_id);
            } else if monster.effects.contains(&effect_id) && crate::effect::parse_effect_change_type(parts[2]) == EffectChangeType::Faded {
                monster.effects.retain(|&id| id != effect_id);
            }
        }

        if let Some(fight) = self.get_current_fight() {
            fight.effect_events.push(effect_event);
        }
    }
}