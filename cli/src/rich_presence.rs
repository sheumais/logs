use std::{fs, io::{BufRead, BufReader, Read, Seek, Write}, path::PathBuf};
use discord_rich_presence::{activity::{self, Assets}, DiscordIpc, DiscordIpcClient};
use parser::{player::Class, zone::is_dungeon};

const CLIENT_ID: &str = "1413962656250986648";
const FALLBACK_IMAGE: &str = "https://images.uesp.net/5/57/ON-map-Aurbis.jpg";

#[derive(Clone)]
enum ZoneDifficulty {
    None,
    Normal,
    Veteran,
    #[allow(dead_code)]
    Hardmode,
}

struct ESOLogsRichPresence {
    pub timestamp_begin_log: Option<u64>,
    pub timestamp_latest_map: Option<u64>,
    pub zone_id: Option<u16>,
    pub zone_difficulty: Option<ZoneDifficulty>,
    pub zone_name: Option<String>,
    pub map_name: Option<String>,
}

impl ESOLogsRichPresence {
    fn new() -> Self {
        ESOLogsRichPresence {
            timestamp_begin_log: None,
            timestamp_latest_map: None,
            zone_id: None,
            zone_difficulty: None,
            zone_name: None,
            map_name: None,
        }
    }
}

pub fn rich_presence_thread() {
    let config_path = dirs::data_local_dir()
        .expect("Failed to resolve local appdata")
        .join("eso-log-tool")
        .join("richpresence.txt");

    let default_path = dirs::document_dir()
        .expect("Failed to find Documents folder")
        .join("Elder Scrolls Online")
        .join("live")
        .join("Logs")
        .join("Encounter.log");

    let input_path: PathBuf = if config_path.exists() {
        let file = fs::File::open(&config_path).expect("Failed to open config file");
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line).expect("Failed to read config line");
        let custom_path = PathBuf::from(line.trim());

        if custom_path.exists() {
            custom_path
        } else {
            log::warn!(
                "Configured path {custom_path:?} does not exist, falling back to default {default_path:?}"
            );
            default_path
        }
    } else {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create config directory");
        }
        let mut f = fs::File::create(&config_path).expect("Failed to create config file");
        writeln!(f, "{}", default_path.display()).expect("Failed to write config file");
        default_path
    };

    let mut input_file = loop {
        match fs::OpenOptions::new().read(true).open(&input_path) {
            Ok(f) => break f,
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_secs(10));
                continue;
            }
        }
    };
    
    let mut pos = {
        let scan_file = std::fs::OpenOptions::new().read(true).open(&input_path)
            .expect("Failed to open file for scanning");
        let mut reader = std::io::BufReader::new(&scan_file);
        let mut pos = 0u64;
        let mut last_begin_log_pos = 0u64;
        loop {
            let start_pos = pos;
            let mut buf = String::new();
            let bytes_read = reader.read_line(&mut buf).expect("read failed");
            if bytes_read == 0 {
                break;
            }
            if buf.contains("END_LOG") {
                last_begin_log_pos = start_pos;
            }
            pos += bytes_read as u64;
        }
        input_file.seek(std::io::SeekFrom::Start(last_begin_log_pos)).expect("seek failed")
    };

    let mut rich_presence = ESOLogsRichPresence::new();

    let mut client = DiscordIpcClient::new(CLIENT_ID);

    let e = client.connect();
    if let Err(e) = e {log::error!("Discord Rich Presence failed to connect: {e:?}"); log::info!("This will occur if Discord is in administrator mode, and ESO Log Tool is not. That may or may not be the cause here."); panic!()}

    loop {
        if input_file
            .seek(std::io::SeekFrom::Start(pos))
            .is_err()
        {
            eprintln!("seek failed");
            break;
        }

        let mut buffer = Vec::new();
        let mut reader = std::io::BufReader::new(&input_file);
        let bytes_read = reader.read_to_end(&mut buffer).unwrap_or(0);

        if bytes_read == 0 {
            std::thread::sleep(std::time::Duration::from_secs(5));
            continue;
        }

        if let Some(last_nl) = buffer.iter().rposition(|&b| b == b'\n') {
            let text = String::from_utf8_lossy(&buffer[..=last_nl]);

            for line in text.lines() {
                let parts: Vec<String> = parser::parse::handle_line(line);

                if parts.len() < 2 {
                    continue;
                }

                match parts[1].as_str() {
                    "BEGIN_LOG" if parts.len() > 3 => {
                        if let Ok(ts) = parts[2].parse::<u64>() {
                            rich_presence.timestamp_begin_log = Some(ts);
                        }
                    }
                    "ZONE_CHANGED" if parts.len() > 4 => {
                        if let Ok(zone_id) = parts[2].parse::<u16>() {
                            rich_presence.zone_id = Some(zone_id);
                        }
                        rich_presence.zone_name = Some(parts[3].trim_matches('"').to_string());
                        rich_presence.zone_difficulty = Some(match parts[4].as_str() {
                            "NORMAL" => ZoneDifficulty::Normal,
                            "VETERAN" => ZoneDifficulty::Veteran,
                            _ => ZoneDifficulty::None,
                        });
                    }
                    "MAP_CHANGED" if parts.len() > 3 => {
                        rich_presence.map_name = Some(parts[3].trim_matches('"').to_string());
                        if let Ok(ts) = parts[0].parse::<u64>() {
                            rich_presence.timestamp_latest_map = Some(ts);
                        }
                    }
                    "END_LOG" => {
                        rich_presence = ESOLogsRichPresence::new();
                    }
                    _ => {}
                }
            }

            pos += (last_nl + 1) as u64;
        }

        let Some(zone_id) = rich_presence.zone_id else {
            let _ = client.clear_activity();
            continue;
        };
        let zone_icon = zone_to_icon(zone_id);
        let is_dungeon = is_dungeon(zone_id);
        if !is_dungeon {
            let _ = client.clear_activity();
            continue;
        }
        let zone_name = rich_presence.zone_name.clone().unwrap_or_else(|| "Unknown Zone".to_string());
        // let map_name = rich_presence.map_name.clone();
        let zone_difficulty = rich_presence.zone_difficulty.clone().unwrap_or(ZoneDifficulty::None);

        let time = match (rich_presence.timestamp_begin_log, rich_presence.timestamp_latest_map) {
            (Some(begin), Some(map_offset)) => Some(((begin + map_offset) / 1000) as i64),
            (Some(begin), None) => Some((begin / 1000) as i64),
            _ => None,
        };
        log::debug!("timestamp: {time:?}");

        // let is_trial = is_trial(zone_id);

        let large_icon = &zone_icon.unwrap_or(FALLBACK_IMAGE.to_string());
        let activity_assets = Assets::new()
            .large_image(large_icon)
            .large_text(&zone_name);

        let difficulty_text = match zone_difficulty {
            ZoneDifficulty::None => "",
            ZoneDifficulty::Normal => "Normal ",
            ZoneDifficulty::Veteran => "Veteran ",
            ZoneDifficulty::Hardmode => "Hardmode ",
        };
        let details_text = format!("{difficulty_text}{zone_name}");

        let mut activity = activity::Activity::new()
            .details(&details_text)
            .activity_type(activity::ActivityType::Playing)
            .status_display_type(activity::StatusDisplayType::Name)
            .assets(activity_assets);

        // #[allow(unused_assignments)]
        // let mut state_text = String::new();
        // if let Some(map) = map_name {
        //     state_text = format!("Inside {}", map);
        //     if map != zone_name {
        //         activity = activity.state(&state_text);
        //     }
        // }

        if let Some(start_time) = time {
            activity = activity.timestamps(activity::Timestamps::new().start(start_time));
        }

        let e = client.set_activity(activity);
        log::debug!("{e:?}");
        
    }
}

/// Maps class to uesp icon url
#[allow(dead_code)]
fn class_to_icon(class: Class) -> Option<String> {
    match class {
        Class::Arcanist => Some("https://images.uesp.net/4/4a/ON-icon-skill-Herald_of_the_Tome-Pragmatic_Fatecarver.png".to_string()),
        Class::Dragonknight => Some("https://images.uesp.net/b/b2/ON-icon-skill-Ardent_Flame-Molten_Whip.png".to_string()),
        Class::Necromancer => Some("https://images.uesp.net/f/fa/ON-icon-skill-Grave_Lord-Venom_Skull.png".to_string()),
        Class::Nightblade => Some("https://images.uesp.net/a/a5/ON-icon-skill-Assassination-Impale.png".to_string()),
        Class::Sorcerer => Some("https://images.uesp.net/7/78/ON-icon-skill-Storm_Calling-Hurricane.png".to_string()),
        Class::Templar => Some("https://images.uesp.net/e/e1/ON-icon-skill-Dawn's_Wrath-Power_of_the_Light.png".to_string()),
        Class::Warden => Some("https://images.uesp.net/d/dd/ON-icon-skill-Animal_Companions-Subterranean_Assault.png".to_string()),
        _ => None,
    }
}

/// Maps zone_id to a uesp url for the zone's loading screen
fn zone_to_icon(zone_id: u16) -> Option<String> {
    match zone_id {
    /* --------- DUNGEONS / TRIALS --------- */
        11 => Some("https://images.uesp.net/b/bc/ON-load-Vaults_of_Madness.jpg".to_string()), // Vaults of Madness
        22 => Some("https://images.uesp.net/f/fb/ON-load-Volenfell.jpg".to_string()), // Volenfell
        31 => Some("https://images.uesp.net/7/7b/ON-load-Selene's_Web_02.jpg".to_string()), // Selene's Web
        38 => Some("https://images.uesp.net/3/31/ON-load-Blackheart_Haven.jpg".to_string()), // Blackheart Haven
        63 => Some("https://images.uesp.net/3/31/ON-load-Darkshade_Caverns.jpg".to_string()), // Darkshade Caverns I
        64 => Some("https://images.uesp.net/d/de/ON-load-Blessed_Crucible.jpg".to_string()), // Blessed Crucible
        126 => Some("https://images.uesp.net/4/4d/ON-load-Elden_Hollow.jpg".to_string()), // Elden Hollow I
        130 => Some("https://images.uesp.net/f/f5/ON-load-Crypt_of_Hearts.jpg".to_string()), // Crypt of Hearts I
        131 => Some("https://images.uesp.net/0/04/ON-load-Tempest_Island.jpg".to_string()), // Tempest Island
        144 => Some("https://images.uesp.net/6/6f/ON-load-Spindleclutch.jpg".to_string()), // Spindleclutch I
        146 => Some("https://images.uesp.net/0/0e/ON-load-Wayrest_Sewers.jpg".to_string()), // Wayrest Sewers I
        148 => Some("https://images.uesp.net/b/b0/ON-load-Arx_Corinium.jpg".to_string()), // Arx Corinium
        176 => Some("https://images.uesp.net/7/7c/ON-load-City_of_Ash.jpg".to_string()), // City of Ash I
        283 => Some("https://images.uesp.net/4/40/ON-load-Fungal_Grotto.jpg".to_string()), // Fungal Grotto I
        380 => Some("https://images.uesp.net/a/ac/ON-load-The_Banished_Cells.jpg".to_string()), // The Banished Cells I
        449 => Some("https://images.uesp.net/6/6e/ON-load-Direfrost_Keep.jpg".to_string()), // Direfrost Keep
        636 => Some("https://images.uesp.net/7/7a/ON-load-Hel_Ra_Citadel.jpg".to_string()), // Hel Ra Citadel
        638 => Some("https://images.uesp.net/f/fc/ON-load-Aetherian_Archive.jpg".to_string()), // Aetherian Archive
        639 => Some("https://images.uesp.net/9/9b/ON-load-Sanctum_Ophidia.jpg".to_string()), // Sanctum Ophidia
        678 => Some("https://images.uesp.net/8/83/ON-load-Imperial_Prison.jpg".to_string()), // Imperial City Prison
        681 => Some("https://images.uesp.net/3/35/ON-load-City_of_Ash_II.jpg".to_string()), // City of Ash II
        688 => Some("https://images.uesp.net/b/b0/ON-load-White-Gold_Tower.jpg".to_string()), // White-Gold Tower
        725 => Some("https://images.uesp.net/6/65/ON-load-Maw_of_Lorkhaj.png".to_string()), // Maw of Lorkhaj
        843 => Some("https://images.uesp.net/9/9e/ON-load-Ruins_of_Mazzatun.png".to_string()), // Ruins of Mazzatun
        848 => Some("https://images.uesp.net/f/f7/ON-load-Cradle_of_Shadows.png".to_string()), // Cradle of Shadows
        930 => Some("https://images.uesp.net/3/31/ON-load-Darkshade_Caverns.jpg".to_string()), // Darkshade Caverns II
        931 => Some("https://images.uesp.net/4/4d/ON-load-Elden_Hollow.jpg".to_string()), // Elden Hollow II
        932 => Some("https://images.uesp.net/6/62/ON-load-Crypt_of_Hearts_II.jpg".to_string()), // Crypt of Hearts II
        933 => Some("https://images.uesp.net/0/0e/ON-load-Wayrest_Sewers.jpg".to_string()), // Wayrest Sewers II
        934 => Some("https://images.uesp.net/4/40/ON-load-Fungal_Grotto.jpg".to_string()), // Fungal Grotto II
        935 => Some("https://images.uesp.net/a/ac/ON-load-The_Banished_Cells.jpg".to_string()), // The Banished Cells II
        936 => Some("https://images.uesp.net/6/6f/ON-load-Spindleclutch.jpg".to_string()), // Spindleclutch II
        973 => Some("https://images.uesp.net/5/5b/ON-load-Bloodroot_Forge.jpg".to_string()), // Bloodroot Forge
        974 => Some("https://images.uesp.net/0/02/ON-load-Falkreath_Hold.jpg".to_string()), // Falkreath Hold
        975 => Some("https://images.uesp.net/5/51/ON-load-Halls_of_Fabrication.jpg".to_string()), // Halls of Fabrication
        1000 => Some("https://images.uesp.net/1/13/ON-load-Asylum_Sanctorium.jpg".to_string()), // Asylum Sanctorium
        1009 => Some("https://images.uesp.net/9/93/ON-load-Fang_Lair.jpg".to_string()), // Fang Lair
        1010 => Some("https://images.uesp.net/2/23/ON-load-Scalecaller_Peak.jpg".to_string()), // Scalecaller Peak
        1051 => Some("https://images.uesp.net/c/cf/ON-load-Cloudrest.jpg".to_string()), // Cloudrest
        1052 => Some("https://images.uesp.net/5/50/ON-load-Moon_Hunter_Keep.jpg".to_string()), // Moon Hunter Keep
        1055 => Some("https://images.uesp.net/9/90/ON-load-March_of_Sacrifices.jpg".to_string()), // March of Sacrifices
        1080 => Some("https://images.uesp.net/b/bb/ON-load-Wrathstone.jpg".to_string()), // Frostvault
        1081 => Some("https://images.uesp.net/2/27/ON-load-Depths_of_Malatar.jpg".to_string()), // Depths of Malatar
        1121 => Some("https://images.uesp.net/e/e8/ON-load-Sunspire.jpg".to_string()), // Sunspire
        1122 => Some("https://images.uesp.net/b/bf/ON-load-Moongrave_Fane.jpg".to_string()), // Moongrave Fane
        1123 => Some("https://images.uesp.net/d/d8/ON-load-Lair_of_Maarselok.jpg".to_string()), // Lair of Maarselok
        1152 => Some("https://images.uesp.net/b/bb/ON-load-Icereach.png".to_string()), // Icereach
        1153 => Some("https://images.uesp.net/2/24/ON-load-Unhallowed_Grave.png".to_string()), // Unhallowed Grave
        1196 => Some("https://images.uesp.net/b/b9/ON-load-Kyne's_Aegis.png".to_string()), // Kyne's Aegis
        1197 => Some("https://images.uesp.net/4/4f/ON-load-Stone_Garden.jpg".to_string()), // Stone Garden
        1201 => Some("https://images.uesp.net/a/af/ON-load-Castle_Thorn.jpg".to_string()), // Castle Thorn
        1228 => Some("https://images.uesp.net/3/37/ON-load-Black_Drake_Villa.png".to_string()), // Black Drake Villa
        1229 => Some("https://images.uesp.net/b/b0/ON-load-The_Cauldron.png".to_string()), // The Cauldron
        1263 => Some("https://images.uesp.net/f/f9/ON-load-Rockgrove.png".to_string()), // Rockgrove
        1267 => Some("https://images.uesp.net/e/e5/ON-load-Red_Petal_Bastion.png".to_string()), // Red Petal Bastion
        1268 => Some("https://images.uesp.net/3/39/ON-load-The_Dread_Cellar.png".to_string()), // The Dread Cellar
        1301 => Some("https://images.uesp.net/8/86/ON-load-Coral_Aerie.png".to_string()), // Coral Aerie
        1302 => Some("https://images.uesp.net/5/5c/ON-load-Shipwright's_Regret.png".to_string()), // Shipwright's Regret
        1344 => Some("https://images.uesp.net/d/d2/ON-load-Dreadsail_Reef.png".to_string()), // Dreadsail Reef
        1360 => Some("https://images.uesp.net/c/c3/ON-load-Earthen_Root_Enclave.jpg".to_string()), // Earthen Root Enclave
        1361 => Some("https://images.uesp.net/9/94/ON-load-Graven_Deep.jpg".to_string()), // Graven Deep
        1389 => Some("https://images.uesp.net/6/63/ON-load-Bal_Sunnar.jpg".to_string()), // Bal Sunnar
        1390 => Some("https://images.uesp.net/7/7c/ON-load-Scrivener's_Hall.jpg".to_string()), // Scrivener's Hall
        1427 => Some("https://images.uesp.net/a/ac/ON-load-Sanity's_Edge.png".to_string()), // Sanity's Edge
        1470 => Some("https://images.uesp.net/3/32/ON-load-Oathsworn_Pit.png".to_string()), // Oathsworn Pit
        1471 => Some("https://images.uesp.net/b/b1/ON-load-Bedlam_Veil.png".to_string()), // Bedlam Veil
        1478 => Some("https://images.uesp.net/4/40/ON-load-Lucent_Citadel.png".to_string()), // Lucent Citadel
        1496 => Some("https://images.uesp.net/c/c9/ON-load-Exiled_Redoubt.png".to_string()), // Exiled Redoubt
        1497 => Some("https://images.uesp.net/8/85/ON-load-Lep_Seclusa.png".to_string()), // Lep Seclusa
        1548 => Some("https://images.uesp.net/3/3d/ON-load-Ossein_Cage.png".to_string()), // Ossein Cage
        1551 => Some("https://images.uesp.net/7/76/ON-load-Naj-Caldeesh.png".to_string()), // Naj Caldeesh
        1552 => Some("https://images.uesp.net/6/60/ON-load-Black_Gem_Foundry.png".to_string()), // Black Gem Foundry

    /* --- Arenas --- */
        635 => Some("https://images.uesp.net/b/b7/ON-load-Dragonstar_Arena.jpg".to_string()), // Dragonstar Arena
        677 => Some("https://images.uesp.net/7/74/ON-load-Maelstrom_Arena.jpg".to_string()), // Maelstrom Arena
        1082 => Some("https://images.uesp.net/7/76/ON-load-Blackrose_Prison.jpg".to_string()), // Blackrose Prison
        1227 => Some("https://images.uesp.net/2/26/ON-load-Vateshran_Hollows.png".to_string()), // Vateshran Hollows
        1436 => Some("https://images.uesp.net/a/a4/ON-load-Infinite_Archive.png".to_string()), // Infinite Archive
        _ => None
    }
}

#[allow(dead_code)]
fn is_trial(zone_id: u16) -> bool {
    match zone_id {
        |  636 // HRC
        |  638 // AA
        |  639 // SO
        |  725 // MOL
        |  975 // HOF
        | 1000 // AS
        | 1051 // CR
        | 1121 // SS
        | 1196 // KA
        | 1263 // RG
        | 1344 // DSR
        | 1427 // SE
        | 1478 // LC
        | 1548 // OC
        => true,
        _ => false,
    }
}