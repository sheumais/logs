use crate::{fight::Fight, player::Player, set::get_item_type_from_hashmap, unit::UnitState};

#[derive(Debug, PartialEq, Clone)]
pub struct Ability {
    pub id: u32,
    pub name: String,
    pub icon: String,
    pub interruptible: bool,
    pub blockable: bool,
    pub scribing: Option<Vec<String>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Effect {
    pub ability: Ability,
    pub stack_count: u16,
    pub effect_type: EffectType,
    pub status_effect_type: StatusEffectType,
    pub synergy: Option<u32>,
}

#[derive(Debug, PartialEq)]
pub struct EffectEvent {
    pub time: u64,
    pub change_type: EffectChangeType,
    pub stack_count: u16,
    pub cast_track_id: u32,
    pub ability_id: u32,
    pub source_unit_state: UnitState,
    pub target_unit_state: UnitState,
    pub player_initiated_remove_cast_track_id: bool,
}

#[derive(Debug, PartialEq)]
pub enum EffectChangeType {
    Faded,
    Gained,
    Updated,
    None,
}

pub fn parse_effect_change_type(string: &str) -> EffectChangeType {
    match string {
        "FADED" => EffectChangeType::Faded,
        "GAINED" => EffectChangeType::Gained,
        "UPDATED" => EffectChangeType::Updated,
        _ => EffectChangeType::None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum EffectType {
    Buff,
    Debuff,
    None,
}

pub fn parse_effect_type(string: &str) -> EffectType {
    match string {
        "BUFF" => EffectType::Buff,
        "DEBUFF" => EffectType::Debuff,
        _ => EffectType::None,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatusEffectType {
    Magic,
    None,
}

pub fn parse_status_effect_type(string: &str) -> StatusEffectType {
    match string {
        "MAGIC" => StatusEffectType::Magic,
        _ => StatusEffectType::None,
    }
}

#[allow(dead_code)]
const ZEN_DEBUFF_ID: &'static u32 = &126597;
#[allow(dead_code)]
const CRITICAL_CHANCE_MAXIMUM: &'static f32 = &21912.00097656250181;
#[allow(dead_code)]
const ENLIVENING_OVERFLOW: &'static u32 = &156008;
#[allow(dead_code)]
const FROM_THE_BRINK: &'static u32 = &156017;

#[allow(dead_code)]
pub fn is_zen_dot(effect: &Effect) -> bool {
    match effect.ability.id {
        // class abilities
        36947 => true, // debilitate
        35336 => true, // lotus fan
        36960 => true, // crippling grasp

        21731 => true, // vampire's bane
        21732 => true, // reflective light

        101944 => true, // growing swarm
        101904 => true, // fetcher infection
        130140 => true, // cutting dive

        20326 => true, // volatile armour
        31898 => true, // burning talons
        31103 => true, // noxious breath
        31104 => true, // engulfing flames
        44369 => true, // venomous claw
        44373 => true, // burning embers

        // pure agony synergy
        // ghostly embrace (2nd circle)

        182989 => true, // fulminating rune
        185840 => true, // rune of displacement

        // weapon abilities
        204009 => true, // tri focus (fire staff)
        38747 => true, // carve
        62712 => true, // frost reach
        38703 => true, // acid spray
        44549 => true, // poison injection 
        85261 => true, // toxic barrage
        44545 => true, // venom arrow
        38841 => true, // rending slashes
        85182 => true, // thrive in chaos
        // rend
        // blood craze


        // world abilities
        137259 => true, // exhilarating drain
        // drain vigor
        // soul splitting trap
        // consuming trap
        // soul assault
        // shatter soul


        // armor sets
        75753 => true, // alkosh (line-breaker)
        97743 => true, // pillar of nirn
        172671 => true, // whorl of the depths
        107203 => true, // arms of relequen

        // guild abilities
        40468 => true, // scalding rune
        40385 => true, // barbed trap
        // lightweight barbed trap
        126374 => true, // degeneration
        126371 => true, // structured entropy
        62314 => true, // dawnbreaker of smiting
        62310 => true, // flawless dawnbreaker

        // other
        18084 => true, // burning
        21929 => true, // poisoned
        148801 => true, // hemorrhaging
        41838 => true, // radiate synergy
        113627 => true, // virulent shot (brp bow)
        79025 => true, // ravage health 3.5s

        _ => {
            if effect.ability.scribing.is_some() {
                effect.ability.scribing.clone().unwrap()[1] == "Lingering Torment"
            } else {
                false
            }
        }
    }
}

#[allow(dead_code)]
/// Calculate percentage of fight that unit had buff
/// 
/// Returns float with value 0 to 1
pub fn buff_uptime_over_fight(buff_id: u32, unit_id: u32, fight: &Fight) -> f32{
    let mut time_with_buff = 0;
    let mut gained_buff_timestamp = fight.start_time;
    let mut has_buff: bool = false;
    for player in &fight.players {
        if player.unit_id == unit_id {
            if player.effects.contains(&buff_id) {
                has_buff = true;
            }
        }
    }
    for monster in &fight.monsters {
        if monster.unit_id == unit_id {
            if monster.effects.contains(&buff_id) {
                has_buff = true;
            }
        }
    }
    for effect_event in &fight.effect_events {
        if effect_event.target_unit_state.unit_id == unit_id && effect_event.ability_id == buff_id {
            if effect_event.change_type == EffectChangeType::Gained && !has_buff {
                gained_buff_timestamp = effect_event.time;
                has_buff = true;
            } else if effect_event.change_type == EffectChangeType::Faded && has_buff {
                let time_difference = effect_event.time - gained_buff_timestamp;
                time_with_buff += time_difference;
                has_buff = false;
            }
        }
    }
    if has_buff {
        let time_difference = fight.end_time - gained_buff_timestamp;
        time_with_buff += time_difference;
    }

    let fight_duration = fight.end_time - fight.start_time;
    if fight_duration > 0 {
        // println!("Time: {} / Duration {}", time_with_buff, fight_duration);
        (time_with_buff as f32) / (fight_duration as f32)
    } else {
        0.0
    }
}

fn update_blockade_ability(ability: &mut Ability, item_type: &crate::set::ItemType) {
    match item_type {
        crate::set::ItemType::FrostStaff => {
            ability.icon = "/esoui/art/icons/ability_destructionstaff_002b.dds".to_string();
            ability.name = "Blockade of Frost".to_string();
        },
        crate::set::ItemType::LightningStaff => {
            ability.icon = "/esoui/art/icons/ability_destructionstaff_003_b.dds".to_string();
            ability.name = "Blockade of Storms".to_string();
        },
        crate::set::ItemType::FireStaff => {
            ability.icon = "/esoui/art/icons/ability_destructionstaff_004_b.dds".to_string();
            ability.name = "Fiery Blockade".to_string();
        },
        _ => {},
    }
}

pub fn destruction_staff_skill_convert(player: &mut Player) {
    let item_type = get_item_type_from_hashmap(player.gear.main_hand.item_id);
    for ability in &mut player.primary_abilities {
        if ability.id == 39011 {
            update_blockade_ability(ability, &item_type);
        }
    }
    let backup_item_type = get_item_type_from_hashmap(player.gear.main_hand_backup.item_id);
    for ability in &mut player.backup_abilities {
        if ability.id == 39011 {
            update_blockade_ability(ability, &backup_item_type);
        }
    }
}