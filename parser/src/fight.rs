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