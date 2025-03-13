use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref SETS: HashMap<u16, &'static str> = parse_set_data_into_hashmap();
}

pub fn parse_set_data_into_hashmap() -> HashMap<u16, &'static str> {
    let mut lookup_table = HashMap::new();
    let data = include_str!("set_data.txt");
    for line in data.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 2 {
            if let Ok(key) = parts[0].parse::<u16>() {
                lookup_table.insert(key, parts[1]);
            }
        }
    }

    lookup_table
}

pub fn get_set_name(id: u16) -> Option<&'static str> {
    SETS.get(&id).cloned()
}

pub fn is_mythic_set(id: u16) -> bool {
    match id {
        501 => true,
        503 => true,
        505 => true,
        519 => true,
        520 => true,
        521 => true,
        575 => true,
        576 => true,
        593 => true,
        594 => true,
        596 => true,
        597 => true,
        625 => true,
        626 => true,
        627 => true,
        654 => true,
        655 => true,
        656 => true,
        657 => true,
        658 => true,
        674 => true,
        675 => true,
        676 => true,
        691 => true,
        692 => true,
        693 => true,
        694 => true,
        760 => true,
        761 => true,
        762 => true,        
        _ => false,
    }
}
