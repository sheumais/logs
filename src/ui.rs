use crate::{event::DamageType, player::{self, ClassId, EnchantType, GearEnchant, GearPiece, GearQuality, GearSlot, GearTrait, Loadout}};
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

    pub fn from_gear_quality(gear_quality: GearQuality) -> Self {
        match gear_quality {
            GearQuality::Trash => Colour::new(195, 195, 195),
            GearQuality::Normal => Colour::new(255, 255, 255),
            GearQuality::Magic => Colour::new(45, 197, 14),
            GearQuality::Arcane => Colour::new(58, 146, 255),
            GearQuality::Artifact => Colour::new(160, 46, 247),
            GearQuality::Legendary => Colour::new(238, 202, 42),
            GearQuality::Mythic => Colour::new(255, 130, 0),
            _ => Colour::new(255,255,255)
        }
    }

    pub fn from_damage_type(damage_type: DamageType) -> Self {
        match damage_type {
            DamageType::Bleed => Colour::new(235, 69, 97),
            DamageType::Cold => Colour::new(143, 242, 255),
            DamageType::Disease => Colour::new(37, 153, 190),
            DamageType::Fire => Colour::new(229, 115, 16),
            DamageType::Generic => Colour::new(191, 191, 191),
            DamageType::Magic => Colour::new(74, 128, 255),
            DamageType::Oblivion => Colour::new(147, 43, 181),
            DamageType::Physical => Colour::new(229, 204, 128),
            DamageType::Poison => Colour::new(209, 250, 153),
            DamageType::Shock => Colour::new(184, 168, 240),
            _ => Colour::new(191, 191, 191)
        }
    }

    pub fn from_class_id(class_id: ClassId) -> Self {
        match class_id {
            ClassId::Arcanist => Colour::new(209, 250, 153),
            ClassId::Dragonknight => Colour::new(229, 115, 16),
            ClassId::Necromancer => Colour::new(147, 43, 181),
            ClassId::Nightblade => Colour::new(232, 155, 155),
            ClassId::Sorcerer => Colour::new(184, 168, 240),
            ClassId::Templar => Colour::new(231, 222, 96),
            ClassId::Warden => Colour::new(14, 120, 21),
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
        DamageType::Generic,
        DamageType::Magic,
        DamageType::Oblivion,
        DamageType::Physical,
        DamageType::Poison,
        DamageType::Shock,
    ];

    let class_ids = [
        ClassId::Arcanist,
        ClassId::Dragonknight,
        ClassId::Necromancer,
        ClassId::Nightblade,
        ClassId::Sorcerer,
        ClassId::Templar,
        ClassId::Warden,
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

    let gear_piece = GearPiece {
        slot: GearSlot::Neck,
        item_id: 123456,
        is_cp: true,
        level: 50,
        gear_trait: GearTrait::Bloodthirsty,
        quality: GearQuality::Artifact,
        set_id: 650,
        enchant: GearEnchant {
            enchant_type: EnchantType::IncreasePhysicalDamage,
            is_enchant_cp: true,
            enchant_level: 50,
            enchant_quality: GearQuality::Legendary,
        },
    };
    println!("{}", gear_piece);
}


impl fmt::Display for Loadout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "head: {}\nshoulders: {}\nchest: {}\nhands: {}\nwaist: {}\nlegs: {}\nfeet: {}\nneck: {}\nring1: {}\nring2: {}\nmain_hand: {}\nmain_hand_backup: {}\noff_hand: {}\noff_hand_backup: {}\n",
            self.head,
            self.shoulders,
            self.chest,
            self.hands,
            self.waist,
            self.legs,
            self.feet,
            self.neck,
            self.ring1,
            self.ring2,
            self.main_hand,
            self.main_hand_backup,
            self.off_hand,
            self.off_hand_backup
        )
    }
}

impl fmt::Display for GearPiece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut colour = Colour::from_gear_quality(self.quality);
        let level = player::veteran_level_to_cp(self.level, self.is_cp);
        let cp_string= if self.is_cp {
            "CP"
        } else {
            "Level"
        };
        if set::is_mythic_set(self.set_id) {colour = Colour::new(255, 130, 0)}
        
        let display_text = format!(
            "{} {:?} {:?} {:?} {} {}",
            cp_string,
            level,
            self.gear_trait,
            self.slot,
            set::get_set_name(self.set_id).unwrap_or_else(|| ""),
            self.enchant, 
        );
        
        let colored_text = foreground_rgb(&display_text, colour);
        write!(f, "{}", colored_text)
    }
}

impl fmt::Display for GearEnchant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let color = Colour::from_gear_quality(self.enchant_quality);
        let level = player::veteran_level_to_cp(self.enchant_level, self.is_enchant_cp);
        let cp_string= if self.is_enchant_cp {
            "CP"
        } else {
            "Level"
        };

        let display_text = format!(
            "{} {:?} {:?}",
            cp_string,
            level,
            self.enchant_type,
        );
        
        let colored_text = foreground_rgb(&display_text, color);
        write!(f, "{}", colored_text)
    }
}