use std::fmt;
use crate::effect::EffectEvent;
use crate::unit::Unit;
use crate::player::Player;
use crate::event::{Event, Cast};

#[derive(Debug, PartialEq)]
pub struct Fight {
    pub id: u16,
    pub name: String,
    pub players: Vec<Player>,
    pub monsters: Vec<Unit>,
    pub start_time: u64,
    pub end_time: u64,
    pub events: Vec<Event>,
    pub casts: Vec<Cast>,
    pub effect_events: Vec<EffectEvent>,
}

impl fmt::Display for Fight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ id: {}, players: {:?}, monsters: {:?}, start_time: {}, end_time: {} }}",
            self.id,
            self.players,
            self.monsters,
            self.start_time,
            self.end_time,
        )
    }
}