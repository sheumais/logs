pub mod effect;
pub mod event;
pub mod fight;
pub mod player;
pub mod set;
pub mod ui;
pub mod unit;
pub mod parse;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    BeginLog,
    EndCombat,
    BeginCombat,
    UnitAdded,
    PlayerInfo,
    AbilityInfo,
    CombatEvent,
    BeginCast,
    EffectChanged,
    MapChanged,
    ZoneChanged,
    EndTrial,
    HealthRegen,
    EffectInfo,
    UnitChanged,
    EndCast,
    Unknown,
}

impl From<&str> for EventType {
    fn from(s: &str) -> Self {
        match s {
            "BEGIN_LOG"      => EventType::BeginLog,
            "END_COMBAT"     => EventType::EndCombat,
            "BEGIN_COMBAT"   => EventType::BeginCombat,
            "UNIT_ADDED"     => EventType::UnitAdded,
            "PLAYER_INFO"    => EventType::PlayerInfo,
            "ABILITY_INFO"   => EventType::AbilityInfo,
            "COMBAT_EVENT"   => EventType::CombatEvent,
            "BEGIN_CAST"     => EventType::BeginCast,
            "EFFECT_CHANGED" => EventType::EffectChanged,
            "MAP_CHANGED"    => EventType::MapChanged,
            "ZONE_CHANGED"   => EventType::ZoneChanged,
            "END_TRIAL"      => EventType::EndTrial,
            "HEALTH_REGEN"   => EventType::HealthRegen,
            "EFFECT_INFO"    => EventType::EffectInfo,
            "UNIT_CHANGED"   => EventType::UnitChanged,
            "END_CAST"       => EventType::EndCast,
            _                => EventType::Unknown,
        }
    }
}

pub enum UnitAddedEventType {
    Player,
    Monster,
    Object,
    Unknown,
}

impl From<&str> for UnitAddedEventType {
    fn from(s: &str) -> Self {
        match s {
            "PLAYER" => UnitAddedEventType::Player,
            "MONSTER" => UnitAddedEventType::Monster,
            "OBJECT" => UnitAddedEventType::Object,
            _ => UnitAddedEventType::Unknown,
        }
    }
}

pub enum EffectChangedEventType {
    Gained,
    Updated,
    Faded,
    Unknown,
}

impl From<&str> for EffectChangedEventType {
    fn from(s: &str) -> Self {
        match s {
            "GAINED" => EffectChangedEventType::Gained,
            "UPDATED" => EffectChangedEventType::Updated,
            "FADED" => EffectChangedEventType::Faded,
            _ => EffectChangedEventType::Unknown,
        }
    }
}