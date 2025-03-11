use std::collections::HashMap;

pub struct Log {
    pub log_epoch: i64,
    pub players: HashMap<u32, crate::player::Player>,
    pub units: HashMap<u32, crate::unit::Unit>,
    pub fights: HashMap<u16, crate::fight::Fight>,
    pub effects: HashMap<u32, crate::effect::Effect>,
}

impl Log {
    pub fn new() -> Self {
        let new_self: Self = Self {
            log_epoch: 0,
            players: HashMap::new(),
            units: HashMap::new(),
            fights: HashMap::new(),
            effects: HashMap::new(),
        };

        new_self
    }

    pub fn parse_line(&mut self, line: &str) {
        let mut in_brackets = false;
        let mut current_segment_start = 0;
        let mut parts = Vec::new();
    
        for (i, char) in line.char_indices() {
            match char {
                '[' => {
                    in_brackets = true;
                    current_segment_start = i + 1;
                }
                ']' => {
                    in_brackets = false;
                    parts.push(&line[current_segment_start..i]);
                    current_segment_start = i + 1;
                }
                ',' if !in_brackets => {
                    parts.push(&line[current_segment_start..i]);
                    current_segment_start = i + 1; 
                }
                _ => {}
            }
        }
    
        if current_segment_start < line.len() {
            parts.push(&line[current_segment_start..]);
        }
        parts.retain(|part| !part.is_empty());
    
        if parts.len() >= 2 {
            match parts[1] {
                "BEGIN_LOG" => self.handle_begin_log(parts),
                "BEGIN_COMBAT" | "END_COMBAT" => self.handle_combat_change(parts),
                "UNIT_ADDED" => self.handle_unit_added(parts),
                "PLAYER_INFO" => self.handle_player_info(parts),
                "ABILITY_INFO" => self.handle_ability_info(parts),
                "EFFECT_INFO" => self.handle_effect_info(parts),
                "COMBAT_EVENT" => self.handle_combat_event(parts),
                "BEGIN_CAST" => self.handle_begin_cast(parts),
                // "END_CAST"
                // "HEALTH_REGEN"
                // "UNIT_CHANGED"
                // "UNIT_REMOVED"
                "EFFECT_CHANGED" => self.handle_effect_changed(parts),
                // "MAP_INFO"
                // "ZONE_INFO"
                // "TRIAL_INIT"
                // "BEGIN_TRIAL"
                // "END_TRIAL"
                _ => {},
            }
        }
    }

    pub fn is_empty(&mut self) -> bool {
        self.fights.is_empty()
    }
    
    fn get_current_fight(&mut self) -> Option<&mut crate::fight::Fight> {
        if let Some(fight) = self.fights.get_mut(&(self.fights.len() as u16 - 1)) {
            if fight.end_time == 0 {
                return Some(fight);
            }
        }
        None
    }

    fn is_true(value: &str) -> bool {
        value == "T"
    }

    fn handle_begin_log(&mut self, parts: Vec<&str>) {
        self.log_epoch = parts[2].parse::<i64>().unwrap();
        let log_version = parts[3];
        if log_version != "15" {
            panic!("Unknown log version: {} (Expected 15)", log_version);
        }
    }

    fn handle_unit_added(&mut self, parts: Vec<&str>) {
        let unit_id: u32 = parts[2].parse::<u32>().unwrap();
        if parts[3] == "PLAYER" {
            let display_name = parts[11].trim_matches('"').to_string();
            let name = parts[10].to_string();
            let player_per_session_id: u32 = parts[5].parse::<u32>().unwrap();
            if display_name != "\"\"" {
                let player = crate::player::Player {
                    unit_id: unit_id,
                    is_local_player: Self::is_true(parts[4]),
                    player_per_session_id: player_per_session_id,
                    class_id: crate::player::match_class(parts[8]),
                    race_id: crate::player::match_race(parts[9]),
                    name: name,
                    display_name: display_name,
                    character_id: parts[12].parse::<i128>().unwrap(),
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
            };
            self.units.insert(unit_id, object);
        }
    }

    fn handle_combat_change(&mut self, parts: Vec<&str>) {
        if parts[1] == "BEGIN_COMBAT" {
            let mut new_fight = crate::fight::Fight {
                id: self.fights.len() as u16,
                name: String::new(),
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
            self.fights.insert(self.fights.len() as u16, new_fight);
        } else if parts[1] == "END_COMBAT" {
            let fight_name = {
                let mut name = "Unknown";
                if let Some(fight) = self.get_current_fight() {
                    let mut unit_ids = Vec::new();
                    for event in &fight.events {
                        if crate::event::does_damage(event.result) {
                            unit_ids.push(event.target_unit_state.unit_id);
                        }
                    }
                    for unit_id in unit_ids {
                        if let Some(unit) = self.units.get(&unit_id) {
                            if unit.unit_type == crate::unit::UnitType::Monster {
                                name = &unit.name;
                                break;
                            }
                        }
                    }
                }
                name.to_string()
            };
        
            if let Some(fight) = self.get_current_fight() {
                fight.end_time = parts[0].parse::<u64>().unwrap();
                fight.name = fight_name;
            }
        }
    }

    // unitId, [longTermEffectAbilityId,...], [longTermEffectStackCounts,...], [<equipmentInfo>,...], [primaryAbilityId,...], [backupAbilityId,...]
    // ["934981", "PLAYER_INFO", "47", "142210,142079,78219,89959,89958,89957,150054,172646,193447,147226,13975,184887,184873,45557,45549,63601,58955,45562,142092,184858,63880,142218,45565,185239,45601,184923,45603,45607,29062,40393,45596,39248,184847,55676,184932,185058,63802,55386,36588,185186,185243,183049,45500,15594,45267,45270,45272,185195,45514,186233,99875,45513,45509,186230,186231,186232", "1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1", "HEAD,94779,T,16,ARMOR_DIVINES,LEGENDARY,270,STAMINA,T,16,LEGENDARY", "NECK,194512,T,16,JEWELRY_BLOODTHIRSTY,LEGENDARY,694,INCREASE_PHYSICAL_DAMAGE,T,16,LEGENDARY", "CHEST,187412,T,16,ARMOR_DIVINES,LEGENDARY,652,STAMINA,T,16,LEGENDARY", "SHOULDERS,147377,T,16,ARMOR_DIVINES,LEGENDARY,127,STAMINA,T,16,LEGENDARY", "MAIN_HAND,87874,T,16,WEAPON_NIRNHONED,LEGENDARY,127,FIERY_WEAPON,T,16,LEGENDARY", "OFF_HAND,87874,T,16,WEAPON_CHARGED,LEGENDARY,127,POISONED_WEAPON,T,16,LEGENDARY", "WAIST,187472,T,16,ARMOR_DIVINES,LEGENDARY,652,STAMINA,T,16,LEGENDARY", "LEGS,187452,T,16,ARMOR_DIVINES,LEGENDARY,652,STAMINA,T,16,LEGENDARY", "FEET,187423,T,16,ARMOR_DIVINES,LEGENDARY,652,STAMINA,T,16,LEGENDARY", "RING1,147512,T,16,JEWELRY_BLOODTHIRSTY,LEGENDARY,127,INCREASE_PHYSICAL_DAMAGE,T,16,LEGENDARY", "RING2,147512,T,16,JEWELRY_BLOODTHIRSTY,LEGENDARY,127,INCREASE_PHYSICAL_DAMAGE,T,16,LEGENDARY", "HAND,187432,T,16,ARMOR_DIVINES,LEGENDARY,652,STAMINA,T,16,LEGENDARY", "BACKUP_MAIN,166198,T,16,WEAPON_INFUSED,LEGENDARY,526,BERSERKER,T,16,LEGENDARY", "40382,40195,38901,183006,186366,40161", "39011,186229,183241,182988,185842,189867"]
    fn handle_player_info(&mut self, parts: Vec<&str>) {
        let unit_id: u32 = parts[2].parse::<u32>().unwrap();
        let effect_id_list: Vec<u32> = parts[3].split(",").map(|x| x.parse::<u32>().unwrap()).collect();
        for (_i, effect_id) in effect_id_list.iter().enumerate() {
            if let Some(player) = self.players.get_mut(&unit_id) {
                // check if player has effect already
                if !player.effects.contains(effect_id) {
                    player.effects.push(*effect_id);
                }
            }
        }

        let gear_parts = parts.len() - 2;
        for i in 5..gear_parts {
            let gear_piece = self.handle_equipment_info(parts[i]);
            if let Some(player) = self.players.get_mut(&unit_id) {
                player.insert_gear_piece(gear_piece);
            }
        }

        let primary_ability_id_list: Vec<u32> = parts[parts.len() - 2].split(",").map(|x| x.parse::<u32>().unwrap_or_default()).collect();
        let backup_ability_id_list: Vec<u32> = parts[parts.len() - 1].split(",").map(|x| x.parse::<u32>().unwrap_or_default()).collect();
        if let Some(player) = self.players.get_mut(&unit_id) {
            player.primary_abilities = primary_ability_id_list;
            player.backup_abilities = backup_ability_id_list;
        }
    }

    fn handle_equipment_info(&mut self, part: &str) -> crate::player::GearPiece {
        let split: Vec<&str> = part.split(",").collect();
        // check all enums for none values, and print what they are
        if crate::player::match_gear_slot(split[0]) == crate::player::GearSlot::None {
            // println!("Unknown gear slot: {}", split[0]);
            println!("{}", part);
        }
        if crate::player::match_gear_trait(split[4]) == crate::player::GearTrait::None && split[4] != "NONE" {
            // println!("Unknown gear trait: {}", split[4]);
            println!("{}", part);
        }
        if crate::player::match_gear_quality(split[5]) == crate::player::GearQuality::None {
            // println!("Unknown gear quality: {}", split[5]);
            println!("{}", part);
        }
        if crate::player::match_enchant_type(split[7]) == crate::player::EnchantType::None {
            // println!("Unknown enchant type: {}", split[7]);
            println!("{}", part);
        }
        let gear_piece = crate::player::GearPiece {
            slot: crate::player::match_gear_slot(split[0]),
            item_id: split[1].parse::<u32>().unwrap(),
            is_cp: Self::is_true(split[2]),
            level: split[3].parse::<u8>().unwrap(),
            trait_id: crate::player::match_gear_trait(split[4]),
            quality: crate::player::match_gear_quality(split[5]),
            set_id: split[6].parse::<u32>().unwrap(),
            enchant: crate::player::GearEnchant {
                enchant_type: crate::player::match_enchant_type(split[7]),
                is_enchant_cp: Self::is_true(split[8]),
                enchant_level: split[9].parse::<u8>().unwrap(),
                enchant_quality: crate::player::match_gear_quality(split[10]),
            },
        };
        gear_piece
    }

    fn handle_ability_info(&mut self, parts: Vec<&str>) {
        let effect_id: u32 = parts[2].parse::<u32>().unwrap(); // abilityId usually unique, but can be reused for scribing abilities
        let name = parts[3].trim_matches('"').to_string();
        let effect = crate::effect::Effect {
            id: effect_id,
            name: name,
            icon: parts[4].to_string(),
            interruptible: Self::is_true(parts[5]),
            blockable: Self::is_true(parts[6]),
            stack_count: 0,
            effect_type: crate::effect::EffectType::None,
            status_effect_type: crate::effect::StatusEffectType::None,
            synergy: None,
        };
        self.effects.insert(effect_id, effect);
    }

    fn handle_effect_info(&mut self, parts: Vec<&str>) {
        let effect_id: u32 = parts[2].parse::<u32>().unwrap();
        let effect = self.effects.get_mut(&effect_id).unwrap();
        effect.effect_type = crate::effect::parse_effect_type(parts[3]);
        effect.status_effect_type = crate::effect::parse_status_effect_type(parts[4]);
        if parts.len() > 6 {
            effect.synergy = parts[6].parse::<u32>().ok();
        }
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

        if crate::event::parse_event_result(parts[2]) == crate::event::EventResult::None {
            println!("Unknown event result: {}", parts[2]);
        }
        let event = crate::event::Event {
            time: parts[0].parse::<u64>().unwrap(),
            result: crate::event::parse_event_result(parts[2]),
            damage_type: crate::event::parse_damage_type(parts[3]),
            power_type: parts[4].parse::<u32>().unwrap(),
            hit_value: parts[5].parse::<u32>().unwrap(),
            overflow: parts[6].parse::<u32>().unwrap(),
            cast_track_id: parts[7].parse::<u32>().unwrap(),
            ability_id: parts[8].parse::<u32>().unwrap(),
            source_unit_state: source_unit_state,
            target_unit_state: target_unit_state,
        };

        if let Some(fight) = self.get_current_fight() {
            fight.events.push(event);
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
        };

        if let Some(fight) = self.get_current_fight() {
            fight.casts.push(cast);
        }
    }

    // how do we track effects over time? 
    // gaining and losing effects is not fight-bound like damage is.
    // therefore a fight-based solution is perhaps not correct.
    // a player may gain a buff shortly before combat starts, but still keep it during the fight.
    // if we don't track out of combat buffs then we would not know the player has this buff.
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

        let player_id = target_unit_state.unit_id;
        let effect_id = effect_event.ability_id;

        if let Some(player) = self.players.get_mut(&player_id) {
            if !player.effects.contains(&effect_id) {
                player.effects.push(effect_id);
            }
        }

        if let Some(fight) = self.get_current_fight() {
            fight.effect_events.push(effect_event);
        }

        // if parts.len() > 27 {
        //     println!("{:?}", parts);
        // }
        // println!("{}", parts.len());
    }
}