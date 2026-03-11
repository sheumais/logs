use esosim::data::skill::SkillLine;

pub fn subclass_to_name(subclass: SkillLine) -> String {
    let name = match subclass {
        SkillLine::DarkMagic => "Dark Magic",
        SkillLine::DaedricSummoning => "Daedric Summoning",
        SkillLine::StormCalling => "Storm Calling",
        SkillLine::AedricSpear => "Aedric Spear",
        SkillLine::DawnsWrath => "Dawn's Wrath",
        SkillLine::RestoringLight => "Restoring Light",
        SkillLine::ArdentFlame => "Ardent Flame",
        SkillLine::DraconicPower => "Draconic Power",
        SkillLine::EarthenHeart => "Earthen Heart",
        SkillLine::Assassination => "Assassination",
        SkillLine::Shadow => "Shadow",
        SkillLine::Siphoning => "Siphoning",
        SkillLine::AnimalCompanions => "Animal Companions",
        SkillLine::GreenBalance => "Green Balance",
        SkillLine::WintersEmbrace => "Winter's Embrace",
        SkillLine::GraveLord => "Grave Lord",
        SkillLine::BoneTyrant => "Bone Tyrant",
        SkillLine::LivingDeath => "Living Death",
        SkillLine::HeraldOfTheTome => "Herald of the Tome",
        SkillLine::SoldierOfApocrypha => "Soldier of Apocrypha",
        SkillLine::CurativeRuneforms => "Curative Runeforms",
        _ => "Unknown",
    };
    return name.to_string()
}

pub fn subclass_to_icon(subclass: SkillLine) -> String {
    let icon = match subclass {
        SkillLine::DarkMagic => "ability_sorcerer_thunderstomp",
        SkillLine::DaedricSummoning => "ability_sorcerer_speedy_familiar",
        SkillLine::StormCalling => "ability_sorcerer_endless_fury",
        SkillLine::AedricSpear => "ability_templar_ripping_spear",
        SkillLine::DawnsWrath => "ability_templar_power_of_the_light",
        SkillLine::RestoringLight => "ability_templar_purifying_ritual",
        SkillLine::ArdentFlame => "ability_dragonknight_001_b",
        SkillLine::DraconicPower => "ability_dragonknight_007_b",
        SkillLine::EarthenHeart => "ability_dragonknight_017a",
        SkillLine::Assassination => "ability_nightblade_017_b",
        SkillLine::Shadow => "ability_nightblade_004_a",
        SkillLine::Siphoning => "ability_nightblade_013_b",
        SkillLine::AnimalCompanions => "ability_warden_013_b",
        SkillLine::GreenBalance => "ability_warden_007_b",
        SkillLine::WintersEmbrace => "ability_warden_003_b",
        SkillLine::GraveLord => "ability_necromancer_001_b",
        SkillLine::BoneTyrant => "ability_necromancer_008_b",
        SkillLine::LivingDeath => "ability_necromancer_013_a",
        SkillLine::HeraldOfTheTome => "ability_arcanist_003_a",
        SkillLine::SoldierOfApocrypha => "ability_arcanist_008_b",
        SkillLine::CurativeRuneforms => "ability_arcanist_013_a",
        _ => "ability_mage_065",
    };
    return icon.to_string()
}