use std::fmt;
use crate::unit::{Unit, UnitType};
use crate::player::Player;

pub struct Fight {
    pub id: i16,
    pub players: Vec<Player>,
    pub monsters: Vec<Unit>,
    pub bosses: Vec<Unit>,
    pub start_time: i64,
    pub end_time: i64,
}

impl fmt::Display for Fight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ id: {}, players: {:?}, monsters: {:?}, bosses: {:?}, start_time: {}, end_time: {} }}",
            self.id,
            self.players.len(),
            self.monsters.len(),
            self.bosses.len(),
            self.start_time,
            self.end_time
        )
    }
}