use crate::unit::UnitState;

#[derive(Debug, PartialEq)]
pub struct Cast {
    pub time: u64,
    pub duration: u32,
    pub channeled: bool,
    pub cast_track_id: u32,
    pub ability_id: u32,
    pub source_unit_state: UnitState,
    pub target_unit_state: UnitState,
    pub interrupt_reason: Option<CastEndReason>,
}

#[derive(Debug, PartialEq)]
pub struct Event {
    pub time: u64,
    pub result: EventResult,
    pub damage_type: DamageType,
    pub power_type: u32,
    pub hit_value: u32,
    pub overflow: u32,
    pub cast_track_id: u32,
    pub ability_id: u32,
    pub source_unit_state: UnitState,
    pub target_unit_state: UnitState,
}

#[derive(Debug, PartialEq)]
pub enum CastEndReason {
    Completed,
    PlayerCancelled,
    Interrupted,
    Failed,
}

pub fn parse_cast_end_reason(end_reason: &str) -> Option<CastEndReason> {
    match end_reason {
        "COMPLETED" => Some(CastEndReason::Completed),
        "PLAYER_CANCELLED" => Some(CastEndReason::PlayerCancelled),
        "INTERRUPTED" => Some(CastEndReason::Interrupted),
        "FAILED" => Some(CastEndReason::Failed),
        _ => None,
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum EventResult {
    AbilityOnCooldown,
    Absorbed,
    AtPetLimit,
    BadTarget,
    BadTargetCombatState,
    BladeTurn,
    Blocked,
    BlockedDamage,
    Busy,
    CannotUse,
    CantSeeTarget,
    CantSwapHotbarIsOverridden,
    CantSwapWhileChangingGear,
    CasterDead,
    Charmed,
    CriticalDamage,
    CriticalHeal,
    Damage,
    DamageShielded,
    Defended,
    Died,
    DiedCompanionXP,
    DiedXP,
    Disarmed,
    Disoriented,
    Dodged,
    DotTick,
    DotTickCritical,
    Failed,
    FailedRequirements,
    FailedSiegeCreationRequirements,
    Falling,
    FallDamage,
    Feared,
    GraveyardDisallowedInInstance,
    GraveyardTooClose,
    Heal,
    HealAbsorbed,
    HotTick,
    HotTickCritical,
    Immune,
    InsufficientResource,
    Intercepted,
    Interrupt,
    Invalid,
    InvalidFixture,
    InvalidJusticeTarget,
    InvalidTerrain,
    InAir,
    InCombat,
    InEnemyKeep,
    InEnemyOutpost,
    InEnemyResource,
    InEnemyTown,
    InHideyHole,
    KilledByDaedricWeapon,
    KilledBySubzone,
    KillingBlow,
    Knockback,
    Levitated,
    MercenaryLimit,
    Miss,
    MissingEmptySoulGem,
    MissingFilledSoulGem,
    MobileGraveyardLimit,
    Mounted,
    MustBeInOwnKeep,
    NotEnoughInventorySpace,
    NotEnoughInventorySpaceSoulGem,
    NotEnoughSpaceForSiege,
    NoLocationFound,
    NoRamAttackableTargetWithinRange,
    NoWeaponsToSwapTo,
    NpcTooClose,
    OffBalance,
    Pacified,
    Parried,
    PartialResist,
    PowerDrain,
    PowerEnergize,
    PreciseDamage,
    Queued,
    RamAttackableTargetsAllDestroyed,
    RamAttackableTargetsAllOccupied,
    Recalling,
    Reflected,
    Reincarnating,
    Resist,
    Resurrect,
    Rooted,
    SelfPlayingTribute,
    SiegeLimit,
    SiegeNotAllowedInZone,
    SiegePackedUp,
    SiegeTooClose,
    Silenced,
    Snared,
    SoulGemResurrectionAccepted,
    Sprinting,
    Staggered,
    Stunned,
    Swimming,
    TargetDead,
    TargetNotInView,
    TargetNotPvpFlagged,
    TargetOutOfRange,
    TargetPlayingTribute,
    TargetTooClose,
    Taunted,
    UnevenTerrain,
    WeaponSwap,
    WreckingDamage,
    WrongWeapon,
}

pub fn parse_event_result(event_result: &str) -> Option<EventResult> {
    match event_result {
        "ABILITY_ON_COOLDOWN" => Some(EventResult::AbilityOnCooldown),
        "ABSORBED" => Some(EventResult::Absorbed),
        "AT_PET_LIMIT" => Some(EventResult::AtPetLimit),
        "BAD_TARGET" => Some(EventResult::BadTarget),
        "BAD_TARGET_COMBAT_STATE" => Some(EventResult::BadTargetCombatState),
        "BLADETURN" => Some(EventResult::BladeTurn),
        "BLOCKED" => Some(EventResult::Blocked),
        "BLOCKED_DAMAGE" => Some(EventResult::BlockedDamage),
        "BUSY" => Some(EventResult::Busy),
        "CANNOT_USE" => Some(EventResult::CannotUse),
        "CANT_SEE_TARGET" => Some(EventResult::CantSeeTarget),
        "CANT_SWAP_HOTBAR_IS_OVERRIDDEN" => Some(EventResult::CantSwapHotbarIsOverridden),
        "CANT_SWAP_WHILE_CHANGING_GEAR" => Some(EventResult::CantSwapWhileChangingGear),
        "CASTER_DEAD" => Some(EventResult::CasterDead),
        "CHARMED" => Some(EventResult::Charmed),
        "CRITICAL_DAMAGE" => Some(EventResult::CriticalDamage),
        "CRITICAL_HEAL" => Some(EventResult::CriticalHeal),
        "DAMAGE" => Some(EventResult::Damage),
        "DAMAGE_SHIELDED" => Some(EventResult::DamageShielded),
        "DEFENDED" => Some(EventResult::Defended),
        "DIED" => Some(EventResult::Died),
        "DIED_COMPANION_XP" => Some(EventResult::DiedCompanionXP),
        "DIED_XP" => Some(EventResult::DiedXP),
        "DISARMED" => Some(EventResult::Disarmed),
        "DISORIENTED" => Some(EventResult::Disoriented),
        "DODGED" => Some(EventResult::Dodged),
        "DOT_TICK" => Some(EventResult::DotTick),
        "DOT_TICK_CRITICAL" => Some(EventResult::DotTickCritical),
        "FAILED" => Some(EventResult::Failed),
        "FAILED_REQUIREMENTS" => Some(EventResult::FailedRequirements),
        "FAILED_SIEGE_CREATION_REQUIREMENTS" => Some(EventResult::FailedSiegeCreationRequirements),
        "FALLING" => Some(EventResult::Falling),
        "FALL_DAMAGE" => Some(EventResult::FallDamage),
        "FEARED" => Some(EventResult::Feared),
        "GRAVEYARD_DISALLOWED_IN_INSTANCE" => Some(EventResult::GraveyardDisallowedInInstance),
        "GRAVEYARD_TOO_CLOSE" => Some(EventResult::GraveyardTooClose),
        "HEAL" => Some(EventResult::Heal),
        "HEAL_ABSORBED" => Some(EventResult::HealAbsorbed),
        "HOT_TICK" => Some(EventResult::HotTick),
        "HOT_TICK_CRITICAL" => Some(EventResult::HotTickCritical),
        "IMMUNE" => Some(EventResult::Immune),
        "INSUFFICIENT_RESOURCE" => Some(EventResult::InsufficientResource),
        "INTERCEPTED" => Some(EventResult::Intercepted),
        "INTERRUPT" => Some(EventResult::Interrupt),
        "INVALID" => Some(EventResult::Invalid),
        "INVALID_FIXTURE" => Some(EventResult::InvalidFixture),
        "INVALID_JUSTICE_TARGET" => Some(EventResult::InvalidJusticeTarget),
        "INVALID_TERRAIN" => Some(EventResult::InvalidTerrain),
        "IN_AIR" => Some(EventResult::InAir),
        "IN_COMBAT" => Some(EventResult::InCombat),
        "IN_ENEMY_KEEP" => Some(EventResult::InEnemyKeep),
        "IN_ENEMY_OUTPOST" => Some(EventResult::InEnemyOutpost),
        "IN_ENEMY_RESOURCE" => Some(EventResult::InEnemyResource),
        "IN_ENEMY_TOWN" => Some(EventResult::InEnemyTown),
        "IN_HIDEYHOLE" => Some(EventResult::InHideyHole),
        "KILLED_BY_DAEDRIC_WEAPON" => Some(EventResult::KilledByDaedricWeapon),
        "KILLED_BY_SUBZONE" => Some(EventResult::KilledBySubzone),
        "KILLING_BLOW" => Some(EventResult::KillingBlow),
        "KNOCKBACK" => Some(EventResult::Knockback),
        "LEVITATED" => Some(EventResult::Levitated),
        "MERCENARY_LIMIT" => Some(EventResult::MercenaryLimit),
        "MISS" => Some(EventResult::Miss),
        "MISSING_EMPTY_SOUL_GEM" => Some(EventResult::MissingEmptySoulGem),
        "MISSING_FILLED_SOUL_GEM" => Some(EventResult::MissingFilledSoulGem),
        "MOBILE_GRAVEYARD_LIMIT" => Some(EventResult::MobileGraveyardLimit),
        "MOUNTED" => Some(EventResult::Mounted),
        "MUST_BE_IN_OWN_KEEP" => Some(EventResult::MustBeInOwnKeep),
        "NOT_ENOUGH_INVENTORY_SPACE" => Some(EventResult::NotEnoughInventorySpace),
        "NOT_ENOUGH_INVENTORY_SPACE_SOUL_GEM" => Some(EventResult::NotEnoughInventorySpaceSoulGem),
        "NOT_ENOUGH_SPACE_FOR_SIEGE" => Some(EventResult::NotEnoughSpaceForSiege),
        "NO_LOCATION_FOUND" => Some(EventResult::NoLocationFound),
        "NO_RAM_ATTACKABLE_TARGET_WITHIN_RANGE" => Some(EventResult::NoRamAttackableTargetWithinRange),
        "NO_WEAPONS_TO_SWAP_TO" => Some(EventResult::NoWeaponsToSwapTo),
        "NPC_TOO_CLOSE" => Some(EventResult::NpcTooClose),
        "OFFBALANCE" => Some(EventResult::OffBalance),
        "PACIFIED" => Some(EventResult::Pacified),
        "PARRIED" => Some(EventResult::Parried),
        "PARTIAL_RESIST" => Some(EventResult::PartialResist),
        "POWER_DRAIN" => Some(EventResult::PowerDrain),
        "POWER_ENERGIZE" => Some(EventResult::PowerEnergize),
        "PRECISE_DAMAGE" => Some(EventResult::PreciseDamage),
        "QUEUED" => Some(EventResult::Queued),
        "RAM_ATTACKABLE_TARGETS_ALL_DESTROYED" => Some(EventResult::RamAttackableTargetsAllDestroyed),
        "RAM_ATTACKABLE_TARGETS_ALL_OCCUPIED" => Some(EventResult::RamAttackableTargetsAllOccupied),
        "RECALLING" => Some(EventResult::Recalling),
        "REFLECTED" => Some(EventResult::Reflected),
        "REINCARNATING" => Some(EventResult::Reincarnating),
        "RESIST" => Some(EventResult::Resist),
        "RESURRECT" => Some(EventResult::Resurrect),
        "ROOTED" => Some(EventResult::Rooted),
        "SELF_PLAYING_TRIBUTE" => Some(EventResult::SelfPlayingTribute),
        "SIEGE_LIMIT" => Some(EventResult::SiegeLimit),
        "SIEGE_NOT_ALLOWED_IN_ZONE" => Some(EventResult::SiegeNotAllowedInZone),
        "SIEGE_PACKED_UP" => Some(EventResult::SiegePackedUp),
        "SIEGE_TOO_CLOSE" => Some(EventResult::SiegeTooClose),
        "SILENCED" => Some(EventResult::Silenced),
        "SNARED" => Some(EventResult::Snared),
        "SOUL_GEM_RESURRECTION_ACCEPTED" => Some(EventResult::SoulGemResurrectionAccepted),
        "SPRINTING" => Some(EventResult::Sprinting),
        "STAGGERED" => Some(EventResult::Staggered),
        "STUNNED" => Some(EventResult::Stunned),
        "SWIMMING" => Some(EventResult::Swimming),
        "TARGET_DEAD" => Some(EventResult::TargetDead),
        "TARGET_NOT_IN_VIEW" => Some(EventResult::TargetNotInView),
        "TARGET_NOT_PVP_FLAGGED" => Some(EventResult::TargetNotPvpFlagged),
        "TARGET_OUT_OF_RANGE" => Some(EventResult::TargetOutOfRange),
        "TARGET_PLAYING_TRIBUTE" => Some(EventResult::TargetPlayingTribute),
        "TARGET_TOO_CLOSE" => Some(EventResult::TargetTooClose),
        "TAUNTED" => Some(EventResult::Taunted),
        "UNEVEN_TERRAIN" => Some(EventResult::UnevenTerrain),
        "WEAPONSWAP" => Some(EventResult::WeaponSwap),
        "WRECKING_DAMAGE" => Some(EventResult::WreckingDamage),
        "WRONG_WEAPON" => Some(EventResult::WrongWeapon),
        _ => None,
    }
}

pub fn is_damage_event(event_result: EventResult) -> bool {
    match event_result {
        EventResult::Damage => true,
        EventResult::BlockedDamage => true,
        EventResult::CriticalDamage => true,
        EventResult::DotTick => true,
        EventResult::DotTickCritical => true,
        _ => false,
    }
}

#[allow(dead_code)]
pub fn is_heal_event(event_result: EventResult) -> bool {
    match event_result {
        EventResult::Heal => true,
        EventResult::HotTick => true,
        EventResult::HotTickCritical => true,
        EventResult::CriticalHeal => true,
        _ => false,
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DamageType {
    Bleed,
    Cold,
    Disease,
    Drown,
    Earth,
    Fire,
    Generic,
    Magic,
    None,
    Oblivion,
    Physical,
    Poison,
    Shock,
}

pub fn parse_damage_type(damage_type: &str) -> DamageType {
    match damage_type {
        "BLEED" => DamageType::Bleed,
        "COLD" => DamageType::Cold,
        "DISEASE" => DamageType::Disease,
        "DROWN" => DamageType::Drown,
        "EARTH" => DamageType::Earth,
        "FIRE" => DamageType::Fire,
        "GENERIC" => DamageType::Generic,
        "MAGIC" => DamageType::Magic,
        "NONE" => DamageType::None,
        "OBLIVION" => DamageType::Oblivion,
        "PHYSICAL" => DamageType::Physical,
        "POISON" => DamageType::Poison,
        "SHOCK" => DamageType::Shock,
        _ => DamageType::None,
    }
}