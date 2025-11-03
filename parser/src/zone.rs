use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ZONE_TO_PARENT: HashMap<u16, u16> = {
        let (zones, _, _) = parse_zone_data();
        zones
    };

    pub static ref ZONE_TO_NAME: HashMap<u16, &'static str> = {
        let (_, names, _) = parse_zone_data();
        names
    };

    pub static ref ZONE_TO_DUNGEON: HashMap<u16, bool> = {
        let (_, _, dungeons) = parse_zone_data();
        dungeons
    };
}

fn parse_zone_data() -> (
    HashMap<u16, u16>,
    HashMap<u16, &'static str>,
    HashMap<u16, bool>
) {
    let mut zone_to_parent = HashMap::new();
    let mut zone_to_name = HashMap::new();
    let mut zone_to_dungeon = HashMap::new();

    let data = include_str!("zone_data.csv");

    for line in data.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() < 4 {
            continue;
        }

        let zone_id: u16 = match parts[0].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let parent_zone_id: u16 = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let zone_name = parts[2].trim();

        let is_dungeon = parts[3].trim().eq_ignore_ascii_case("x");

        zone_to_parent.insert(zone_id, parent_zone_id);
        zone_to_name.insert(zone_id, zone_name);
        zone_to_dungeon.insert(zone_id, is_dungeon);
    }

    (zone_to_parent, zone_to_name, zone_to_dungeon)
}

pub fn print_parent_zones() {
    let mut unique_parents = std::collections::BTreeSet::new();

    for parent in ZONE_TO_PARENT.values() {
        unique_parents.insert(parent);
    }

    for parent_id in &unique_parents {
        if let Some(name) = ZONE_TO_NAME.get(parent_id) {
            println!("{parent_id} => , // {name}");
        } else {
            println!("{parent_id}");
        }
    }
}

pub fn print_dungeon_zones() {
    let mut dungeons = std::collections::BTreeSet::new();

    for (zone_id, is_dungeon) in ZONE_TO_DUNGEON.iter() {
        if *is_dungeon {
            dungeons.insert(*zone_id);
        }
    }

    for zone_id in &dungeons {
        if let Some(name) = ZONE_TO_NAME.get(zone_id) {
            println!("{zone_id} => , // {name}");
        } else {
            println!("{zone_id}");
        }
    }
}

pub fn is_dungeon(zone_id: u16) -> bool {
    *ZONE_TO_DUNGEON.get(&zone_id).unwrap_or(&false)
}