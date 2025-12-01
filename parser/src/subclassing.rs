#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subclass {
    DarkMagic = 41,
    DaedricSummoning = 42,
    StormCalling = 43,

    AedricSpear = 22,
    DawnsWrath = 27,
    RestoringLight = 28,

    ArdentFlame = 35,
    DraconicPower = 36,
    EarthenHeart = 37,

    Assassination = 38,
    Shadow = 39,
    Siphoning = 40,

    AnimalCompanions = 127,
    GreenBalance = 128,
    WintersEmbrace = 129,

    GraveLord = 131,
    BoneTyrant = 132,
    LivingDeath = 133,

    HeraldOfTheTome = 218,
    SoldierOfApocrypha = 219,
    CurativeRuneforms = 220,
}

pub fn subclass_to_name(subclass: Subclass) -> String {
    match subclass {
        Subclass::DarkMagic => "Dark Magic".to_string(),
        Subclass::DaedricSummoning => "Daedric Summoning".to_string(),
        Subclass::StormCalling => "Storm Calling".to_string(),
        Subclass::AedricSpear => "Aedric Spear".to_string(),
        Subclass::DawnsWrath => "Dawn's Wrath".to_string(),
        Subclass::RestoringLight => "Restoring Light".to_string(),
        Subclass::ArdentFlame => "Ardent Flame".to_string(),
        Subclass::DraconicPower => "Draconic Power".to_string(),
        Subclass::EarthenHeart => "Earthen Heart".to_string(),
        Subclass::Assassination => "Assassination".to_string(),
        Subclass::Shadow => "Shadow".to_string(),
        Subclass::Siphoning => "Siphoning".to_string(),
        Subclass::AnimalCompanions => "Animal Companions".to_string(),
        Subclass::GreenBalance => "Green Balance".to_string(),
        Subclass::WintersEmbrace => "Winter's Embrace".to_string(),
        Subclass::GraveLord => "Grave Lord".to_string(),
        Subclass::BoneTyrant => "Bone Tyrant".to_string(),
        Subclass::LivingDeath => "Living Death".to_string(),
        Subclass::HeraldOfTheTome => "Herald of the Tome".to_string(),
        Subclass::SoldierOfApocrypha => "Soldier of Apocrypha".to_string(),
        Subclass::CurativeRuneforms => "Curative Runeforms".to_string(),
    }
}

pub fn subclass_to_icon(subclass: Subclass) -> String {
    match subclass {
        Subclass::DarkMagic => "ability_sorcerer_thunderstomp".to_string(),
        Subclass::DaedricSummoning => "ability_sorcerer_speedy_familiar".to_string(),
        Subclass::StormCalling => "ability_sorcerer_endless_fury".to_string(),
        Subclass::AedricSpear => "ability_templar_ripping_spear".to_string(),
        Subclass::DawnsWrath => "ability_templar_power_of_the_light".to_string(),
        Subclass::RestoringLight => "ability_templar_purifying_ritual".to_string(),
        Subclass::ArdentFlame => "ability_dragonknight_001_b".to_string(),
        Subclass::DraconicPower => "ability_dragonknight_007_b".to_string(),
        Subclass::EarthenHeart => "ability_dragonknight_017a".to_string(),
        Subclass::Assassination => "ability_nightblade_017_b".to_string(),
        Subclass::Shadow => "ability_nightblade_004_a".to_string(),
        Subclass::Siphoning => "ability_nightblade_013_b".to_string(),
        Subclass::AnimalCompanions => "ability_warden_013_b".to_string(),
        Subclass::GreenBalance => "ability_warden_007_b".to_string(),
        Subclass::WintersEmbrace => "ability_warden_003_b".to_string(),
        Subclass::GraveLord => "ability_necromancer_001_b".to_string(),
        Subclass::BoneTyrant => "ability_necromancer_008_b".to_string(),
        Subclass::LivingDeath => "ability_necromancer_013_a".to_string(),
        Subclass::HeraldOfTheTome => "ability_arcanist_003_a".to_string(),
        Subclass::SoldierOfApocrypha => "ability_arcanist_008_b".to_string(),
        Subclass::CurativeRuneforms => "ability_arcanist_013_a".to_string(),
    }
}

/// If player has a buff with the given ability id, what subclass does it guarantee they have (if any)?
pub fn ability_id_to_subclassing(ability_id: u32) -> Option<Subclass> {
    match ability_id {
        
        UNHOLY_KNOWLEDGE | BLOOD_MAGIC | PERSISTENCE | EXPLOITATION
        => Some(Subclass::DarkMagic),
        
        REBATE | POWER_STONE | DAEDRIC_PROTECTION | EXPERT_SUMMONER
        => Some(Subclass::DaedricSummoning),
        
        CAPACITOR | ENERGIZED | EXPERT_MAGE
        => Some(Subclass::StormCalling),
        
        PIERCING_SPEAR | SPEAR_WALL | BURNING_LIGHT | BALANCED_WARRIOR
        => Some(Subclass::AedricSpear),
        
        ENDURING_RAYS | PRISM | ILLUMINATE | RESTORING_SPIRIT
        => Some(Subclass::DawnsWrath),
        
        SACRED_GROUND | LIGHT_WEAVER | MASTER_RITUALIST
        => Some(Subclass::RestoringLight),
        
        COMBUSTION | WARMTH
        => Some(Subclass::ArdentFlame),
        
        IRON_SKIN | ELDER_DRAGON | SCALED_ARMOUR
        => Some(Subclass::DraconicPower),
        
        ETERNAL_MOUNTAIN | BATTLE_ROAR
        => Some(Subclass::EarthenHeart),
        
        MASTER_ASSASSIN | EXECUTIONER | PRESSURE_POINTS | HEMORRHAGE
        => Some(Subclass::Assassination),
        
        REFRESHING_SHADOWS | SHADOW_BARRIER | DARK_VIGOR | DARK_VEIL
        => Some(Subclass::Shadow),
        
        CATALYST | SOUL_SIPHONER
        => Some(Subclass::Siphoning),
        
        BOND_WITH_NATURE | SAVAGE_BEAST | FLOURISH | ADVANCED_SPECIES
        => Some(Subclass::AnimalCompanions),
        
        ACCELERATED_GROWTH | NATURES_GIFT | EMERALD_MOSS | MATURATION
        => Some(Subclass::GreenBalance),
        
        FROZEN_ARMOUR | ICY_AURA | PIERCING_COLD
        => Some(Subclass::WintersEmbrace),
        
        REUSABLE_PARTS | DISMEMBER | RAPID_ROD
        => Some(Subclass::GraveLord),
        
        DEATH_GLEANING | HEALTH_AVARICE | LAST_GASP
        => Some(Subclass::BoneTyrant),
        
        CURATIVE_CURSE | NEAR_DEATH_EXPERIENCE | CORPSE_CONSUMPTION | UNDEAD_CONFEDERATE
        => Some(Subclass::LivingDeath),
        
        FATED_FORTUNE | HARNESSED_QUINTESSENCE | PSYCHIC_LESION | SPLINTERED_SECRETS
        => Some(Subclass::HeraldOfTheTome),
        
        AEGIS_OF_THE_UNSEEN | WELLSPRING_OF_THE_ABYSS | CIRCUMVENTED_FATE | IMPLACABLE_OUTCOME
        => Some(Subclass::SoldierOfApocrypha),
    
        HEALING_TIDES | HIDEOUS_CLARITY | ERUDITION | INTRICATE_RUNEFORMS 
        => Some(Subclass::CurativeRuneforms),
        
        _ => None,
    }
}

// Passives
const ETERNAL_MOUNTAIN: u32 = 44996;
const PSYCHIC_LESION: u32 = 184873;
const HARNESSED_QUINTESSENCE: u32 = 184858;
const FATED_FORTUNE: u32 = 184847;
const WARMTH: u32 = 45012;
const REUSABLE_PARTS: u32 = 116188;
const PRESSURE_POINTS: u32 = 45053;
const EXECUTIONER: u32 = 45048;
const COMBUSTION: u32 = 45011;
const NATURES_GIFT: u32 = 85879;
const LAST_GASP: u32 = 116272;
const HEALTH_AVARICE: u32 = 116270;
const SACRED_GROUND: u32 = 45207;
const MASTER_RITUALIST: u32 = 45202;
const INTRICATE_RUNEFORMS: u32 = 185195;
const HIDEOUS_CLARITY: u32 = 185243;
const HEALING_TIDES: u32 = 185186;
const ERUDITION: u32 = 185239;
const WELLSPRING_OF_THE_ABYSS: u32 = 185036;
const REBATE: u32 = 45198;
const POWER_STONE: u32 = 45196;
const IMPLACABLE_OUTCOME: u32 = 185058;
const EXPERT_SUMMONER: u32 = 45199;
const DAEDRIC_PROTECTION: u32 = 45200;
const CIRCUMVENTED_FATE: u32 = 184932;
const SPLINTERED_SECRETS: u32 = 184887;
const DEATH_GLEANING: u32 = 116235;
const DISMEMBER: u32 = 116194;
const CATALYST: u32 = 45135;
const LIGHT_WEAVER: u32 = 45208;
const SOUL_SIPHONER: u32 = 45155;
const ACCELERATED_GROWTH: u32 = 85883;
const EMERALD_MOSS: u32 = 85877;
const MATURATION: u32 = 85881;
const BATTLE_ROAR: u32 = 44984;
const ICY_AURA: u32 = 86194;
const AEGIS_OF_THE_UNSEEN: u32 = 184923;
const PIERCING_COLD: u32 = 86196;
const RAPID_ROD: u32 = 116201;
const MASTER_ASSASSIN: u32 = 45038;
const HEMORRHAGE: u32 = 45060;
const CAPACITOR: u32 = 45188;
const DARK_VEIL: u32 = 45115;
const ENERGIZED: u32 = 45190;
const EXPERT_MAGE: u32 = 45195;
const FLOURISH: u32 = 86067;
const REFRESHING_SHADOWS: u32 = 45103;
const SAVAGE_BEAST: u32 = 86063;
const ADVANCED_SPECIES: u32 = 86069;
const BOND_WITH_NATURE: u32 = 86065;
const DARK_VIGOR: u32 = 45084;
const SHADOW_BARRIER: u32 = 45071;
const BALANCED_WARRIOR: u32 = 44732;
const BURNING_LIGHT: u32 = 44730;
const PIERCING_SPEAR: u32 = 44046;
const ELDER_DRAGON: u32 = 44951;
const SCALED_ARMOUR: u32 = 44953;
const IRON_SKIN: u32 = 44922;
const SPEAR_WALL: u32 = 44721;
const CORPSE_CONSUMPTION: u32 = 116285;
const CURATIVE_CURSE: u32 = 116287;
const ENDURING_RAYS: u32 = 45214;
const FROZEN_ARMOUR: u32 = 86190;
const ILLUMINATE: u32 = 45215;
const NEAR_DEATH_EXPERIENCE: u32 = 116275;
const PRISM: u32 = 45216;
const RESTORING_SPIRIT: u32 = 45212;
const UNDEAD_CONFEDERATE: u32 = 116283;
const EXPLOITATION: u32 = 45181;
const BLOOD_MAGIC: u32 = 45172;
const PERSISTENCE: u32 = 45165;
const UNHOLY_KNOWLEDGE: u32 = 45176;