use std::fmt;
use crate::unit::Unit;
use crate::player::Player;
use crate::event::{Event, Cast};

#[derive(Debug, PartialEq)]
pub struct Fight {
    pub id: u16,
    pub players: Vec<Player>,
    pub monsters: Vec<Unit>,
    pub bosses: Vec<Unit>,
    pub start_time: u64,
    pub end_time: u64,
    pub events: Vec<Event>,
    pub casts: Vec<Cast>
}

impl fmt::Display for Fight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ id: {}, players: {:?}, monsters: {:?}, bosses: {:?}, start_time: {}, end_time: {}, events: {:?} }}",
            self.id,
            self.players,
            self.monsters,
            self.bosses,
            self.start_time,
            self.end_time,
            self.events,
        )
    }
}