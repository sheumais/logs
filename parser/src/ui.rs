use crate::{event::DamageType, player::{self, Class, EnchantType, GearEnchant, GearPiece, GearQuality, GearTrait, Loadout}};
use std::fmt;
use crate::set;

pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl Colour {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Colour { r, g, b }
    }

    fn from_tuple(color: (u8, u8, u8)) -> Self {
        Colour::new(color.0, color.1, color.2)
    }

    const TRASH: (u8, u8, u8) = (195, 195, 195);
    const NORMAL: (u8, u8, u8) = (255, 255, 255);
    const MAGIC: (u8, u8, u8) = (45, 197, 14);
    const ARCANE: (u8, u8, u8) = (58, 146, 255);
    const ARTIFACT: (u8, u8, u8) = (160, 46, 247);
    const LEGENDARY: (u8, u8, u8) = (238, 202, 42);
    const MYTHIC: (u8, u8, u8) = (255, 130, 0);

    const BLEED: (u8, u8, u8) = (235, 69, 97);
    const COLD: (u8, u8, u8) = (143, 242, 255);
    const DISEASE: (u8, u8, u8) = (37, 153, 190);
    const FIRE: (u8, u8, u8) = (229, 115, 16);
    const GENERIC: (u8, u8, u8) = (191, 191, 191);
    const MAGIC_DAMAGE: (u8, u8, u8) = (74, 128, 255);
    const OBLIVION: (u8, u8, u8) = (147, 43, 181);
    const PHYSICAL: (u8, u8, u8) = (229, 204, 128);
    const POISON: (u8, u8, u8) = (209, 250, 153);
    const SHOCK: (u8, u8, u8) = (184, 168, 240);

    const ARCANIST: (u8, u8, u8) = (209, 250, 153);
    const DRAGONKNIGHT: (u8, u8, u8) = (229, 115, 16);
    const NECROMANCER: (u8, u8, u8) = (147, 43, 181);
    const NIGHTBLADE: (u8, u8, u8) = (232, 155, 155);
    const SORCERER: (u8, u8, u8) = (184, 168, 240);
    const TEMPLAR: (u8, u8, u8) = (231, 222, 96);
    const WARDEN: (u8, u8, u8) = (14, 120, 21);

    pub fn from_gear_quality(gear_quality: GearQuality) -> Self {
        match gear_quality {
            GearQuality::Trash => Colour::from_tuple(Self::TRASH),
            GearQuality::Normal => Colour::from_tuple(Self::NORMAL),
            GearQuality::Magic => Colour::from_tuple(Self::MAGIC),
            GearQuality::Arcane => Colour::from_tuple(Self::ARCANE),
            GearQuality::Artifact => Colour::from_tuple(Self::ARTIFACT),
            GearQuality::Legendary => Colour::from_tuple(Self::LEGENDARY),
            GearQuality::Mythic => Colour::from_tuple(Self::MYTHIC),
            _ => Colour::from_tuple(Self::NORMAL),
        }
    }

    pub fn from_damage_type(damage_type: DamageType) -> Self {
        match damage_type {
            DamageType::Bleed => Colour::from_tuple(Self::BLEED),
            DamageType::Cold => Colour::from_tuple(Self::COLD),
            DamageType::Disease => Colour::from_tuple(Self::DISEASE),
            DamageType::Fire => Colour::from_tuple(Self::FIRE),
            DamageType::Heal => Colour::from_tuple(Self::GENERIC),
            DamageType::Magic => Colour::from_tuple(Self::MAGIC_DAMAGE),
            DamageType::Oblivion => Colour::from_tuple(Self::OBLIVION),
            DamageType::Physical => Colour::from_tuple(Self::PHYSICAL),
            DamageType::Poison => Colour::from_tuple(Self::POISON),
            DamageType::Shock => Colour::from_tuple(Self::SHOCK),
            _ => Colour::from_tuple(Self::GENERIC),
        }
    }

    pub fn from_class_id(class_id: Class) -> Self {
        match class_id {
            Class::Arcanist => Colour::from_tuple(Self::ARCANIST),
            Class::Dragonknight => Colour::from_tuple(Self::DRAGONKNIGHT),
            Class::Necromancer => Colour::from_tuple(Self::NECROMANCER),
            Class::Nightblade => Colour::from_tuple(Self::NIGHTBLADE),
            Class::Sorcerer => Colour::from_tuple(Self::SORCERER),
            Class::Templar => Colour::from_tuple(Self::TEMPLAR),
            Class::Warden => Colour::from_tuple(Self::WARDEN),
            _ => Colour::from_tuple(Self::GENERIC),
        }
    }

    pub fn to_ansi_escape(&self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }
}

pub fn foreground_rgb(text: &str, colour: Colour) -> String {
    let colour_escape = colour.to_ansi_escape();
    format!("{}{}{}", colour_escape, text, "\x1b[0m")
}

pub fn print_colour_test() {
    let gear_qualities = [
        GearQuality::Trash,
        GearQuality::Normal,
        GearQuality::Magic,
        GearQuality::Arcane,
        GearQuality::Artifact,
        GearQuality::Legendary,
        GearQuality::Mythic,
    ];

    let damage_types = [
        DamageType::Bleed,
        DamageType::Cold,
        DamageType::Disease,
        DamageType::Fire,
        DamageType::Heal,
        DamageType::Magic,
        DamageType::Oblivion,
        DamageType::Physical,
        DamageType::Poison,
        DamageType::Shock,
    ];

    let class_ids = [
        Class::Arcanist,
        Class::Dragonknight,
        Class::Necromancer,
        Class::Nightblade,
        Class::Sorcerer,
        Class::Templar,
        Class::Warden,
    ];

    println!("Gear Quality Colours:");
    for gear_quality in gear_qualities.iter() {
        let colour = Colour::from_gear_quality(*gear_quality);
        let colour_name = format!("{:?}", gear_quality);
        let output = foreground_rgb(&colour_name, colour);
        println!("{}", output);
    }

    println!("\nDamage Type Colours:");
    for damage_type in damage_types.iter() {
        let colour = Colour::from_damage_type(*damage_type);
        let colour_name = format!("{:?}", damage_type);
        let output = foreground_rgb(&colour_name, colour);
        println!("{}", output);
    }

    println!("\nClass Colours:");
    for class_id in class_ids.iter() {
        let colour = Colour::from_class_id(*class_id);
        let colour_name = format!("{:?}", class_id);
        let output = foreground_rgb(&colour_name, colour);
        println!("{}", output);
    }
}


impl fmt::Display for Loadout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let gear_pieces = vec![
            &self.head,
            &self.shoulders,
            &self.chest,
            &self.hands,
            &self.waist,
            &self.legs,
            &self.feet,
            &self.necklace,
            &self.ring1,
            &self.ring2,
            &self.main_hand,
            &self.off_hand,
            &self.poison,
            &self.main_hand_backup,
            &self.off_hand_backup,
            &self.backup_poison,
        ];

        let gear_list: Vec<String> = gear_pieces
            .iter()
            .filter(|&&gear| gear != &player::empty_gear_piece())
            .map(|&gear| gear.to_string())
            .collect();

        let result = gear_list.join("\n");
        write!(f, "{}", result)
    }
}

impl fmt::Display for GearPiece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut colour = Colour::from_gear_quality(self.quality);
        let level = player::veteran_level_to_cp(self.level, self.is_cp);
        let cp_string = if self.is_cp {"CP"} else {"Level "};
        if set::is_mythic_set(self.set_id) {colour = Colour::from_tuple(Colour::MYTHIC);}
        let set_name = set::get_set_name(self.set_id);

        let mut display_text = format!("");

        display_text.push_str(&format!("{:?} ", self.slot));

        if !player::is_maximum_item_level(self.level, self.is_cp) {
            display_text.push_str(&format!("{}{:?} ", cp_string, level));
        }

        if let Some(name) = set_name {
            display_text.push_str(&format!("{} ", name));
        }

        if set::get_item_type_from_hashmap(self.item_id) != crate::set::ItemType::Unknown {
            display_text.push_str(&format!("{} ", set::get_item_type_name(set::get_item_type_from_hashmap(self.item_id))));
        }

        if self.gear_trait != GearTrait::None {
            display_text.push_str(&format!("{:?} ", self.gear_trait));
        }

        if self.enchant.is_some() {
            let enchant_unwrapped = self.enchant.clone().unwrap();
            if enchant_unwrapped.enchant_type != EnchantType::Invalid {
                display_text.push_str(&format!("{} ", enchant_unwrapped));
            }
        }

        // display_text.push_str(&format!("({})", self.item_id));

        let colored_text = foreground_rgb(&display_text, colour);
        write!(f, "{}", colored_text)
    }
}

impl fmt::Display for GearEnchant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let color = Colour::from_gear_quality(self.enchant_quality);
        let level = player::veteran_level_to_cp(self.enchant_level, self.is_cp);
        let cp_string = if self.is_cp {"CP"} else {"Level "};

        let mut display_text = format!(
            "{:?}",
            self.enchant_type,
        );

        if !player::is_maximum_item_level(self.enchant_level, self.is_cp) {
            display_text.push_str(&format!(" {}{:?}", cp_string, level));
        }

        let colored_text = foreground_rgb(&display_text, color);
        write!(f, "{}", colored_text)
    }
}