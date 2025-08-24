use std::collections::HashMap;

use crate::{effect::{self, Ability, Effect}, player::{self, empty_gear_piece, GearPiece, Player}, unit::{self, Unit, UnitType}};

pub fn is_true(value: &str) -> bool {
    value == "T"
}

pub fn unit_state(parts: &[String], start_index: usize) -> unit::UnitState {
    if start_index >= parts.len() {
        println!("Impossible unit state: {:?}", parts);
    }
    let parse_value = |s: &str| s.parse::<u32>().unwrap_or(0);
    let parse_pair = |s: &str| {
        let mut split = s.split('/');
        (
            split.next().map_or(0, parse_value),
            split.next().map_or(0, parse_value),
        )
    };

    unit::UnitState {
        unit_id: parts[start_index].parse::<u32>().unwrap_or(0),
        health: parse_pair(&parts[start_index + 1]).0,
        max_health: parse_pair(&parts[start_index + 1]).1,
        magicka: parse_pair(&parts[start_index + 2]).0,
        max_magicka: parse_pair(&parts[start_index + 2]).1,
        stamina: parse_pair(&parts[start_index + 3]).0,
        max_stamina: parse_pair(&parts[start_index + 3]).1,
        ultimate: parse_pair(&parts[start_index + 4]).0,
        max_ultimate: parse_pair(&parts[start_index + 4]).1,
        werewolf: parse_pair(&parts[start_index + 5]).0,
        werewolf_max: parse_pair(&parts[start_index + 5]).1,
        shield: parts[start_index + 6].parse::<u32>().unwrap_or(0),
        map_x: parts[start_index + 7].parse::<f32>().unwrap_or(0.0),
        map_y: parts[start_index + 8].parse::<f32>().unwrap_or(0.0),
        heading: parts[start_index + 9].parse::<f32>().unwrap_or(0.0),
    }
}

pub fn player(parts: &[String]) -> Player {
    let unit_id: u32 = parts[2].parse().unwrap();
    Player {
        unit_id,
        is_local_player: is_true(&parts[4]),
        player_per_session_id: parts[5].parse().unwrap(),
        class_id: player::match_class(&parts[8]),
        race_id: player::match_race(&parts[9]),
        name: parts[10].trim_matches('"').to_string(),
        display_name: parts[11].trim_matches('"').to_string(),
        character_id: parts[12].parse().unwrap(),
        level: parts[13].parse().unwrap(),
        champion_points: parts[14].parse().unwrap(),
        is_grouped_with_local_player: is_true(&parts[17]),
        unit_state: unit::blank_unit_state(),
        effects: Vec::new(),
        gear: player::empty_loadout(),
        primary_abilities: Vec::new(),
        backup_abilities: Vec::new(),
    }
}

pub fn monster(parts: &[String]) -> Unit {
    let unit_id: u32 = parts[2].parse().unwrap();
    Unit {
        unit_id,
        unit_type: UnitType::Monster,
        monster_id: parts[6].parse().unwrap(),
        is_boss: is_true(&parts[7]),
        name: parts[10].trim_matches('"').to_string(),
        level: parts[13].parse().unwrap(),
        champion_points: parts[14].parse().unwrap(),
        owner_unit_id: parts[15].parse().unwrap(),
        reaction: unit::match_reaction(&parts[16]),
        unit_state: unit::blank_unit_state(),
        effects: Vec::new(),
    }
}

pub fn monster_updated(parts: &[String], unit: Unit) -> Unit {
    Unit {
        unit_id: parts[2].parse().unwrap(),
        unit_type: unit.unit_type,
        monster_id: unit.monster_id,
        is_boss: unit.is_boss,
        name: parts[5].trim_matches('"').to_string(),
        level: parts[8].parse().unwrap(),
        champion_points: parts[9].parse().unwrap(),
        owner_unit_id: parts[10].parse().unwrap(),
        reaction: unit::match_reaction(&parts[11]),
        unit_state: unit.unit_state,
        effects: unit.effects,
    }
}

pub fn object(parts: &[String]) -> Unit {
    let unit_id: u32 = parts[2].parse().unwrap();
    Unit {
        unit_id,
        unit_type: UnitType::Object,
        monster_id: 0,
        is_boss: false,
        name: parts[10].trim_matches('"').to_string(),
        level: parts[13].parse().unwrap(),
        champion_points: parts[14].parse().unwrap(),
        owner_unit_id: parts[15].parse().unwrap(),
        reaction: unit::match_reaction(&parts[16]),
        unit_state: unit::blank_unit_state(),
        effects: Vec::new(),
    }
}

pub fn gear_piece(part: &str) -> GearPiece {
    let split: Vec<&str> = part.split(',').collect();
    if split.len() < 3 {println!("Gear piece malformed: {:?}", split); return empty_gear_piece()}
    if player::match_gear_trait(split[4]) == player::GearTrait::None && split[4] != "NONE" {
        println!("Unknown gear trait: {part}");
    }

    let enchant = if split.len() > 7  && split[7] != "INVALID" {
        let et = player::match_enchant_type(split[7]);
        if et == player::EnchantType::None && split[7] != "NONE" {
            println!("Unknown enchant type: {part}");
        }

        Some(player::GearEnchant {
            enchant_type: et,
            is_cp: split.get(8).map_or(false, |v| is_true(v)),
            enchant_level: split.get(9).and_then(|v| v.parse().ok()).unwrap_or(0),
            enchant_quality: split
                .get(10)
                .map(|v| player::match_gear_quality(v))
                .unwrap_or(player::GearQuality::None),
        })
    } else {
        None
    };

    GearPiece {
        slot: player::match_gear_slot(split[0]),
        item_id: split[1].parse().unwrap_or(0),
        is_cp: is_true(split[2]),
        level: split[3].parse().unwrap_or(0),
        gear_trait: player::match_gear_trait(split[4]),
        quality: player::match_gear_quality(split[5]),
        set_id: split[6].parse().unwrap_or(0),
        enchant,
    }
}

pub fn ability(parts: &[String]) -> Ability {
    let id: u32 = parts[2].parse().unwrap();
    Ability {
        id,
        name: parts[3].trim_matches('"').to_string(),
        icon: parts[4]
            .trim_matches('"')
            .split('/')
            .last()
            .unwrap()
            .replace(".dds", ".png"),
        interruptible: is_true(&parts[5]),
        blockable: is_true(&parts[6]),
        scribing: if parts.len() == 10 {
            Some((7..10).map(|i| parts[i].trim_matches('"').to_owned()).collect())
        } else {
            None
        },
    }
}

pub fn effect(parts: &[String], ability_lookup: &HashMap<u32, Ability>) -> Effect {
    let effect_id: u32 = parts[2].parse().unwrap();
    let ability = ability_lookup
        .get(&effect_id)
        .expect("ABILITY_INFO must precede EFFECT_INFO")
        .clone();
    Effect {
        ability,
        stack_count: 0,
        effect_type: effect::parse_effect_type(&parts[3]),
        status_effect_type: effect::parse_status_effect_type(&parts[4]),
        synergy: if parts.len() > 6 {
            parts[6].parse().ok()
        } else {
            None
        },
    }
}

pub fn handle_line(line: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut bracket_level = 0;
    let mut in_quotes = false;
    
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    
    for i in 0..len {
        let ch = chars[i];
        
        match ch {
            '"' => {
                let mut backslashes = 0;
                let mut j = i;
                while j > 0 && chars[j - 1] == '\\' {
                    backslashes += 1;
                    j -= 1;
                }
                if backslashes % 2 == 0 {
                    in_quotes = !in_quotes;
                }
            }
            '[' if !in_quotes => {
                bracket_level += 1;
            }
            ']' if !in_quotes => {
                if bracket_level > 0 {
                    bracket_level -= 1;
                }
            }
            ',' if bracket_level == 0 && !in_quotes => {
                if start < i {
                    process_segment(&chars, start, i, &mut result);
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    
    if start < len {
        process_segment(&chars, start, len, &mut result);
    }
    
    result
}

fn process_segment(chars: &[char], start: usize, end: usize, result: &mut Vec<String>) {
    let mut real_start = start;
    let mut real_end = end;
    
    while real_start < real_end && chars[real_start].is_whitespace() {
        real_start += 1;
    }
    while real_end > real_start && chars[real_end - 1].is_whitespace() {
        real_end -= 1;
    }
    
    if real_start >= real_end {
        return;
    }
    
    if chars[real_start] == '[' && chars[real_end - 1] == ']' {
        process_array(&chars[real_start + 1..real_end - 1], result);
    } else {
        let value: String = chars[real_start..real_end].iter().collect();
        result.push(value);
    }
}

fn process_array(chars: &[char], result: &mut Vec<String>) {
    if chars.is_empty() {
        result.push(String::new());
        return;
    }
    
    let has_nested_brackets = chars.iter().any(|&c| c == '[');
    
    if !has_nested_brackets {
        let content: String = chars.iter().collect();
        result.push(content);
        return;
    }
    
    let mut start = 0;
    let mut bracket_level = 0;
    let mut in_quotes = false;
    let mut found_any_segments = false;
    
    for i in 0..chars.len() {
        let ch = chars[i];
        
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            '[' if !in_quotes => {
                bracket_level += 1;
            }
            ']' if !in_quotes => {
                bracket_level -= 1;
            }
            ',' if bracket_level == 0 && !in_quotes => {
                process_nested_segment(&chars[start..i], result);
                found_any_segments = true;
                start = i + 1;
            }
            _ => {}
        }
    }
    
    if start < chars.len() {
        process_nested_segment(&chars[start..chars.len()], result);
        found_any_segments = true;
    }
    
    if !found_any_segments {
        process_nested_segment(&chars[0..chars.len()], result);
    }
}

fn process_nested_segment(chars: &[char], result: &mut Vec<String>) {
    let mut start = 0;
    let mut end = chars.len();
    
    while start < end && chars[start].is_whitespace() {
        start += 1;
    }
    while end > start && chars[end - 1].is_whitespace() {
        end -= 1;
    }
    
    if start >= end {
        result.push(String::new());
        return;
    }
    
    let trimmed = &chars[start..end];
    
    if trimmed[0] == '[' && trimmed[trimmed.len() - 1] == ']' {
        process_array(&trimmed[1..trimmed.len() - 1], result);
    } else {
        let content: String = trimmed.iter().collect();
        result.push(content);
    }
}

#[cfg(test)]
mod tests {
    use crate::{effect::{EffectType, StatusEffectType}, player::{veteran_level_to_cp, Class, EnchantType, GearEnchant, GearQuality, GearSlot, GearTrait, Race}, set::{get_item_type_from_hashmap, is_mythic_set, ItemType}};

    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_is_true() {
        assert!(is_true("T"));
        assert!(!is_true("F"));
        assert!(!is_true("anything_else"));
    }

    #[test]
    fn test_unit_state_parsing() {
        let parts = vec![
            "409425".to_string(), "BEGIN_CAST".to_string(), "0".to_string(), "F".to_string(), "6871610".to_string(), "28541".to_string(),
            "52".to_string(), "15734/24002".to_string(), "13722/15101".to_string(), "18278/27548".to_string(), "223/500".to_string(), "1000/1000".to_string(),
            "2747".to_string(), "0.4029".to_string(), "0.4727".to_string(), "2.0490".to_string(), "*".to_string()
        ];

        let state = unit_state(&parts, 6);
        assert_eq!(state.unit_id, 52);
        assert_eq!(state.health, 15734);
        assert_eq!(state.max_health, 24002);
        assert_eq!(state.magicka, 13722);
        assert_eq!(state.max_magicka, 15101);
        assert_eq!(state.stamina, 18278);
        assert_eq!(state.max_stamina, 27548);
        assert_eq!(state.ultimate, 223);
        assert_eq!(state.max_ultimate, 500);
        assert_eq!(state.werewolf, 1000);
        assert_eq!(state.werewolf_max, 1000);
        assert_eq!(state.shield, 2747);
        assert_eq!(state.map_x, 0.4029);
        assert_eq!(state.map_y, 0.4727);
        assert_eq!(state.heading, 2.0490);
    }

    #[test]
    fn test_player_parsing() {
        let parts = vec![
            "7".to_string(), "UNIT_ADDED".to_string(), "1".to_string(), "PLAYER".to_string(),
            "T".to_string(), "1".to_string(), "0".to_string(), "F".to_string(),
            "5".to_string(), "5".to_string(), "\"Ulfsild's Contingency\"".to_string(),
            "\"@TheMrPancake\"".to_string(), "11353088777847095529".to_string(), "50".to_string(),
            "1833".to_string(), "0".to_string(), "PLAYER_ALLY".to_string(), "T".to_string()
        ];

        let player = player(&parts);
        assert_eq!(player.unit_id, 1);
        assert!(player.is_local_player);
        assert_eq!(player.player_per_session_id, 1);
        assert_eq!(player.class_id, Class::Necromancer);
        assert_eq!(player.race_id, Race::Nord);
        assert_eq!(player.name, "Ulfsild's Contingency");
        assert_eq!(player.display_name, "@TheMrPancake");
        assert_eq!(player.character_id, 11353088777847095529);
        assert_eq!(player.level, 50);
        assert_eq!(player.champion_points, 1833);
        assert!(player.is_grouped_with_local_player);
    }

    #[test]
    fn test_gear_piece_parsing() {
        let gear_str = "CHEST,194509,T,16,ARMOR_NIRNHONED,LEGENDARY,691,PRISMATIC_DEFENSE,T,16,LEGENDARY".to_string();
        let gear = gear_piece(&gear_str);
        assert_eq!(gear.slot, GearSlot::Chest);
        assert_eq!(gear.item_id, 194509);
        assert!(gear.is_cp);
        assert_eq!(veteran_level_to_cp(gear.level, gear.is_cp), 160);
        assert_eq!(gear.gear_trait, GearTrait::Nirnhoned);
        assert_eq!(gear.quality, GearQuality::Legendary);
        assert_eq!(gear.set_id, 691);
        assert_eq!(is_mythic_set(gear.set_id), true);
        assert!(gear.enchant.is_some());
    }

    #[test]
    fn test_ability_parsing() {
        let parts = vec![
            "218302".to_string(), "ABILITY_INFO".to_string(), "183430".to_string(), "\"Runic Sunder\"".to_string(),
            "\"/esoui/art/icons/ability_arcanist_007_a.dds\"".to_string(), "F".to_string(), "T".to_string()
        ];

        let ability = ability(&parts);
        assert_eq!(ability.id, 183430);
        assert_eq!(ability.name, "Runic Sunder");
        assert_eq!(ability.icon, "ability_arcanist_007_a.png");
        assert!(!ability.interruptible);
        assert!(ability.blockable);
        assert!(ability.scribing.is_none());
    }

    #[test]
    fn test_ability_scribing_parsing() {
        let parts = vec![
            "218302".to_string(), "ABILITY_INFO".to_string(), "217784".to_string(), "\"Leashing Soul\"".to_string(),
            "\"/esoui/art/icons/ability_grimoire_soulmagic1.dds\"".to_string(), "F".to_string(), "T".to_string(),
            "\"Pull\"".to_string(), "\"Druid's Resurgence\"".to_string(), "\"Cowardice\"".to_string(),
        ];

        let ability = ability(&parts);
        assert_eq!(ability.id, 217784);
        assert_eq!(ability.name, "Leashing Soul");
        assert_eq!(ability.icon, "ability_grimoire_soulmagic1.png");
        assert!(!ability.interruptible);
        assert!(ability.blockable);
        assert_eq!(ability.scribing.unwrap(), vec!["Pull", "Druid's Resurgence", "Cowardice"]);
    }

    #[test]
    fn test_effect_info() {
        let mut abilities = HashMap::new();
        abilities.insert(85843, Ability {
            id: 85843,
            name: "Harvest".to_string(),
            icon: "ability_warden_007.png".to_string(),
            interruptible: false,
            blockable: true,
            scribing: None,
        });

        let parts = vec![
            "194552".to_string(), "EFFECT_INFO".to_string(), "85843".to_string(), "BUFF".to_string(),
            "NONE".to_string(), "NEVER".to_string(), "85572".to_string()
        ];

        let effect = effect(&parts, &abilities);
        assert_eq!(effect.ability.id, 85843);
        assert_eq!(effect.effect_type, EffectType::Buff);
        assert_eq!(effect.status_effect_type, StatusEffectType::None);
        assert_eq!(effect.synergy, Some(85572));
    }

        #[test]
    fn test_raw_line_parsing() {
        let line = r#"218304,PLAYER_INFO,38,[142210,142079,84732,84731],[1,1,1,1],[[[HEAD,183807,T,16],[NECK,187650,T,16],[CHEST,111911,T,16]]],0,40058"#;
        
        let result = handle_line(line);
        
        assert_eq!(result[0], "218304");
        assert_eq!(result[1], "PLAYER_INFO");
        assert_eq!(result[2], "38");
        assert_eq!(result[3], "142210,142079,84732,84731");
        assert_eq!(result[4], "1,1,1,1");
        assert_eq!(result[5], "HEAD,183807,T,16");
        assert_eq!(result[6], "NECK,187650,T,16");
        assert_eq!(result[7], "CHEST,111911,T,16");
        assert_eq!(result[8], "0");
        assert_eq!(result[9], "40058");
    }

    #[test]
    fn test_simple_vs_nested_vs_complex() {
        let simple_line = "[A,B,C,D]";
        let simple_result = handle_line(simple_line);
        assert_eq!(simple_result, vec!["A,B,C,D"]);
        
        let nested_line = "[[A,B],[C,D]]";
        let nested_result = handle_line(nested_line);
        assert_eq!(nested_result, vec!["A,B", "C,D"]);

        let complex_line = r#"[[A,B],["C","D,"]]"#;
        let complex_result = handle_line(complex_line);
        assert_eq!(complex_result, vec!["A,B", r#""C","D,""#]);
    }

    #[test]
    fn test_empty_player() {
        let line = r#"1,PLAYER_INFO,1,[],[],[],[],[]"#;
        
        let result = handle_line(line);
        
        assert_eq!(result[0], "1");
        assert_eq!(result[1], "PLAYER_INFO");
        assert_eq!(result[2], "1");
        assert_eq!(result[3], "");
        assert_eq!(result[4], "");
        assert_eq!(result[5], "");
        assert_eq!(result[6], "");
        assert_eq!(result[7], "");
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn test_extreme_player_definition() {
        let line = r#"3597,PLAYER_INFO,1,[142079,78219,72824,150054,147459,46751,39248,35770,46041,33090,70390,117848,45301,63802,13984,34741,61930,135397,203342,215493,122586,120017,61685,120023,61662,120028,61691,120029,61666,120008,61744,120015,109966,177885,147417,93109,120020,88490,120021,120025,120013,61747,177886,120024,120026],[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,0,1,1,2,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],[[HEAD,185032,T,8,ARMOR_PROSPEROUS,ARCANE,640,INVALID,F,0,NORMAL],[NECK,171437,T,16,JEWELRY_ARCANE,LEGENDARY,576,INCREASE_BASH_DAMAGE,F,35,ARCANE],[CHEST,45095,T,16,ARMOR_REINFORCED,LEGENDARY,0,PRISMATIC_DEFENSE,F,5,LEGENDARY],[SHOULDERS,56058,F,12,ARMOR_NIRNHONED,MAGIC,0,INVALID,F,0,NORMAL],[OFF_HAND,184873,T,6,ARMOR_DIVINES,ARCANE,640,INVALID,F,0,NORMAL],[WAIST,184888,F,1,ARMOR_DIVINES,NORMAL,640,INVALID,F,0,NORMAL],[LEGS,45169,T,1,ARMOR_TRAINING,ARCANE,0,MAGICKA,F,35,MAGIC],[FEET,45061,F,50,ARMOR_IMPENETRABLE,ARTIFACT,0,MAGICKA,F,35,ARCANE],[COSTUME,55262,F,1,NONE,ARCANE,0,INVALID,F,0,NORMAL],[RING1,139657,F,1,JEWELRY_BLOODTHIRSTY,ARTIFACT,0,INVALID,F,0,NORMAL],[RING2,44904,F,0,NONE,LEGENDARY,0,INVALID,F,0,NORMAL],[BACKUP_POISON,79690,F,1,NONE,LEGENDARY,0,INVALID,F,0,NORMAL],[HAND,185058,F,28,ARMOR_STURDY,NORMAL,640,HEALTH,F,30,NORMAL],[BACKUP_MAIN,185007,T,12,WEAPON_CHARGED,MAGIC,640,DAMAGE_SHIELD,F,35,ARTIFACT],[BACKUP_OFF,184897,T,12,WEAPON_PRECISE,NORMAL,640,FROZEN_WEAPON,F,35,ARCANE]],[25267,61919,34843,36901,25380,113105],[36935,35419,61507,34727,36028]"#;
        let result = handle_line(line);
        assert_eq!(result.len(), 22);
        {
            let gear = gear_piece(&result[5]);
            assert_eq!(gear.slot, GearSlot::Head);
            assert_eq!(gear.item_id, 185032);
            assert!(gear.is_cp);
            assert_eq!(veteran_level_to_cp(gear.level, gear.is_cp), 80);
            assert_eq!(gear.gear_trait, GearTrait::Invigorating);
            assert_eq!(gear.quality, GearQuality::Arcane);
            assert_eq!(gear.set_id, 640);
            assert!(gear.enchant.is_none());
            assert_eq!(get_item_type_from_hashmap(gear.item_id.clone()), ItemType::Medium);
        }

        {
            let gear = gear_piece(&result[6]);
            assert_eq!(gear.slot, GearSlot::Necklace);
            assert_eq!(gear.item_id, 171437);
            assert!(gear.is_cp);
            assert_eq!(gear.level, 16);
            assert_eq!(gear.gear_trait, GearTrait::Arcane);
            assert_eq!(gear.quality, GearQuality::Legendary);
            assert_eq!(gear.set_id, 576);
            assert_eq!(gear.enchant, Some(GearEnchant {
                enchant_type: EnchantType::IncreaseBashDamage,
                is_cp: false,
                enchant_level: 35,
                enchant_quality: GearQuality::Arcane
            }));
        }

        {
            let gear = gear_piece(&result[7]);
            assert_eq!(gear.slot, GearSlot::Chest);
            assert_eq!(gear.item_id, 45095);
            assert!(gear.is_cp);
            assert_eq!(gear.gear_trait, GearTrait::Reinforced);
            assert_eq!(gear.quality, GearQuality::Legendary);
            assert_eq!(gear.set_id, 0);
            assert_eq!(gear.enchant, Some(GearEnchant {
                enchant_type: EnchantType::PrismaticDefense,
                is_cp: false,
                enchant_level: 5,
                enchant_quality: GearQuality::Legendary
            }));
            assert_eq!(get_item_type_from_hashmap(gear.item_id.clone()), ItemType::Heavy);
        }

        {
            let gear = gear_piece(&result[8]);
            assert_eq!(gear.slot, GearSlot::Shoulders);
            assert_eq!(gear.item_id, 56058);
            assert!(!gear.is_cp);
            assert_eq!(gear.gear_trait, GearTrait::Nirnhoned);
            assert_eq!(gear.quality, GearQuality::Magic);
            assert_eq!(gear.set_id, 0);
            assert!(gear.enchant.is_none());
            assert_eq!(get_item_type_from_hashmap(gear.item_id.clone()), ItemType::Medium);
        }

        {
            let gear = gear_piece(&result[9]);
            assert_eq!(gear.slot, GearSlot::OffHand);
            assert_eq!(gear.item_id, 184873);
            assert!(gear.is_cp);
            assert_eq!(gear.gear_trait, GearTrait::Divines);
            assert_eq!(gear.quality, GearQuality::Arcane);
            assert_eq!(gear.set_id, 640);
            assert_eq!(get_item_type_from_hashmap(gear.item_id.clone()), ItemType::Shield);
        }

        {
            let gear = gear_piece(&result[10]);
            assert_eq!(gear.slot, GearSlot::Waist);
            assert_eq!(gear.item_id, 184888);
            assert!(!gear.is_cp);
            assert_eq!(gear.gear_trait, GearTrait::Divines);
            assert_eq!(gear.quality, GearQuality::Normal);
            assert_eq!(gear.set_id, 640);
            assert_eq!(get_item_type_from_hashmap(gear.item_id.clone()), ItemType::Light);
        }

        {
            let gear = gear_piece(&result[11]);
            assert_eq!(gear.slot, GearSlot::Legs);
            assert_eq!(gear.item_id, 45169);
            assert!(gear.is_cp);
            assert_eq!(gear.gear_trait, GearTrait::Training);
            assert_eq!(gear.quality, GearQuality::Arcane);
            assert_eq!(gear.enchant, Some(GearEnchant {
                enchant_type: EnchantType::Magicka,
                is_cp: false,
                enchant_level: 35,
                enchant_quality: GearQuality::Magic
            }));
            assert_eq!(get_item_type_from_hashmap(gear.item_id.clone()), ItemType::Heavy);
        }

        {
            let gear = gear_piece(&result[12]);
            assert_eq!(gear.slot, GearSlot::Feet);
            assert_eq!(gear.item_id, 45061);
            assert!(!gear.is_cp);
            assert_eq!(gear.gear_trait, GearTrait::Impenetrable);
            assert_eq!(gear.quality, GearQuality::Artifact);
            assert_eq!(gear.enchant, Some(GearEnchant {
                enchant_type: EnchantType::Magicka,
                is_cp: false,
                enchant_level: 35,
                enchant_quality: GearQuality::Arcane
            }));
            assert_eq!(get_item_type_from_hashmap(gear.item_id.clone()), ItemType::Heavy);
        }

        {
            let gear = gear_piece(&result[13]);
            assert_eq!(gear.slot, GearSlot::Costume);
            assert_eq!(gear.item_id, 55262);
            assert_eq!(gear.gear_trait, GearTrait::None);
            assert_eq!(gear.quality, GearQuality::Arcane);
            assert!(gear.enchant.is_none());
        }

        {
            let gear = gear_piece(&result[14]);
            assert_eq!(gear.slot, GearSlot::Ring1);
            assert_eq!(gear.item_id, 139657);
            assert_eq!(gear.gear_trait, GearTrait::Bloodthirsty);
            assert_eq!(gear.quality, GearQuality::Artifact);
        }

        {
            let gear = gear_piece(&result[15]);
            assert_eq!(gear.slot, GearSlot::Ring2);
            assert_eq!(gear.item_id, 44904);
            assert_eq!(gear.gear_trait, GearTrait::None);
            assert_eq!(gear.quality, GearQuality::Legendary);
            assert_eq!(get_item_type_from_hashmap(gear.item_id.clone()), ItemType::Mara);
        }

        {
            let gear = gear_piece(&result[16]);
            assert_eq!(gear.slot, GearSlot::BackupPoison);
            assert_eq!(gear.item_id, 79690);
            assert_eq!(gear.gear_trait, GearTrait::None);
            assert_eq!(gear.quality, GearQuality::Legendary);
        }

        {
            let gear = gear_piece(&result[17]);
            assert_eq!(gear.slot, GearSlot::Hands);
            assert_eq!(gear.item_id, 185058);
            assert_eq!(gear.gear_trait, GearTrait::Sturdy);
            assert_eq!(gear.quality, GearQuality::Normal);
            assert_eq!(gear.enchant, Some(GearEnchant {
                enchant_type: EnchantType::Health,
                is_cp: false,
                enchant_level: 30,
                enchant_quality: GearQuality::Normal
            }));
            assert_eq!(get_item_type_from_hashmap(gear.item_id.clone()), ItemType::Light);
        }

        {
            let gear = gear_piece(&result[18]);
            assert_eq!(gear.slot, GearSlot::MainHandBackup);
            assert_eq!(gear.item_id, 185007);
            assert!(gear.is_cp);
            assert_eq!(gear.gear_trait, GearTrait::Charged);
            assert_eq!(gear.quality, GearQuality::Magic);
            assert_eq!(gear.enchant, Some(GearEnchant {
                enchant_type: EnchantType::DamageShield,
                is_cp: false,
                enchant_level: 35,
                enchant_quality: GearQuality::Artifact
            }));
            assert_eq!(get_item_type_from_hashmap(gear.item_id.clone()), ItemType::Dagger);
        }

        {
            let gear = gear_piece(&result[19]);
            assert_eq!(gear.slot, GearSlot::OffHandBackup);
            assert_eq!(gear.item_id, 184897);
            assert!(gear.is_cp);
            assert_eq!(gear.gear_trait, GearTrait::Precise);
            assert_eq!(gear.quality, GearQuality::Normal);
            assert_eq!(gear.enchant, Some(GearEnchant {
                enchant_type: EnchantType::FrozenWeapon,
                is_cp: false,
                enchant_level: 35,
                enchant_quality: GearQuality::Arcane
            }));
            assert_eq!(get_item_type_from_hashmap(gear.item_id.clone()), ItemType::Mace);
        }
    }
}