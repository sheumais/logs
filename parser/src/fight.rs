use crate::effect::EffectEvent;
use crate::unit::Unit;
use crate::player::Player;
use crate::event::{is_damage_event, Cast, Event};

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

impl Fight {
    pub fn get_average_boss_health_percentage(&self) -> Option<f32> {
        let mut bosses = Vec::new();
        for unit in &self.monsters {
            if unit.is_boss {
                bosses.push(unit.unit_id.clone());
            }
        }
        if bosses.is_empty() {
            return None;
        }

        use std::collections::HashMap;
        let mut boss_health: HashMap<_, f32> = bosses.iter().map(|id| (id, 100.0)).collect();

        for event in &self.events {
            if is_damage_event(event.result) {
                let target_unit_id = &event.target_unit_state.unit_id;
                if bosses.contains(target_unit_id) {
                    let health = (event.target_unit_state.health as f32 / event.target_unit_state.max_health as f32) * 100.0;
                    boss_health.insert(target_unit_id, health);
                }
            }
        }

        let sum: f32 = boss_health.values().sum();
        let avg = sum / boss_health.len() as f32;
        Some(avg)
    }
}