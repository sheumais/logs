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
                "Configured path {:?} does not exist, falling back to default {:?}",
                custom_path,
                default_path
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
    match e {
        Err(e) => {log::error!("Discord Rich Presence failed to connect: {:?}", e); log::info!("This will occur if Discord is in administrator mode, and ESO Log Tool is not. That may or may not be the cause here."); panic!()},
        Ok(_) => {},
    }

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
                let parts: Vec<String> = parser::parse::handle_line(&line);

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
                        let _ = client.clear_activity();
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
        let map_name = rich_presence.map_name.clone();
        let zone_difficulty = rich_presence.zone_difficulty.clone().unwrap_or_else(|| ZoneDifficulty::None);

        let time = match (rich_presence.timestamp_begin_log, rich_presence.timestamp_latest_map) {
            (Some(begin), Some(map_offset)) => Some(((begin + map_offset) / 1000) as i64),
            (Some(begin), None) => Some((begin / 1000) as i64),
            _ => None,
        };
        log::debug!("timestamp: {:?}", time);

        // let is_trial = is_trial(zone_id);

        let large_icon = &zone_icon.unwrap_or(FALLBACK_IMAGE.to_string());
        let activity_assets = Assets::new()
            .large_image(&large_icon)
            .large_text(&zone_name);

        let difficulty_text = match zone_difficulty {
            ZoneDifficulty::None => "",
            ZoneDifficulty::Normal => "Normal ",
            ZoneDifficulty::Veteran => "Veteran ",
            ZoneDifficulty::Hardmode => "Hardmode ",
        };
        let details_text = format!("{}{}", difficulty_text, zone_name);

        let mut activity = activity::Activity::new()
            .details(&details_text)
            .activity_type(activity::ActivityType::Playing)
            .status_display_type(activity::StatusDisplayType::Name)
            .assets(activity_assets);

        #[allow(unused_assignments)]
        let mut state_text = String::new();
        if let Some(map) = map_name {
            state_text = format!("Inside {}", map);
            if map != zone_name {
                activity = activity.state(&state_text);
            }
        }



        if let Some(start_time) = time {
            activity = activity.timestamps(activity::Timestamps::new().start(start_time));
        }

        let e = client.set_activity(activity);
        log::debug!("{:?}", e);
        
    }
}

/// Maps class to icon
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

/// Maps zone_id to a uesp image link for the zone's loading screen if it's a parent zone, or is a trial/dungeon
fn zone_to_icon(zone_id: u16) -> Option<String> {
    match zone_id {
    /* --------- DUNGEONS / TRIALS --------- */
        // 11 => Some("".to_string(), // Vaults of Madness
        // 22 => Some("".to_string(), // Volenfell
        // 31 => Some("".to_string(), // Selene's Web
        // 38 => Some("".to_string(), // Blackheart Haven
        // 63 => Some("".to_string(), // Darkshade Caverns I
        // 64 => Some("".to_string(), // Blessed Crucible
        // 126 => Some("".to_string(), // Elden Hollow I
        // 130 => Some("".to_string(), // Crypt of Hearts I
        // 131 => Some("".to_string(), // Tempest Island
        // 144 => Some("".to_string(), // Spindleclutch I
        // 146 => Some("".to_string(), // Wayrest Sewers I
        // 148 => Some("".to_string(), // Arx Corinium
        // 176 => Some("".to_string(), // City of Ash I
        // 283 => Some("".to_string(), // Fungal Grotto I
        // 380 => Some("".to_string(), // The Banished Cells I
        // 449 => Some("".to_string(), // Direfrost Keep
        // 636 => Some("".to_string(), // Hel Ra Citadel
        // 638 => Some("".to_string(), // Aetherian Archive
        // 639 => Some("".to_string(), // Sanctum Ophidia
        // 678 => Some("".to_string(), // Imperial City Prison
        // 681 => Some("".to_string(), // City of Ash II
        // 688 => Some("".to_string(), // White-Gold Tower
        // 725 => Some("".to_string(), // Maw of Lorkhaj
        // 843 => Some("".to_string(), // Ruins of Mazzatun
        // 848 => Some("".to_string(), // Cradle of Shadows
        // 930 => Some("".to_string(), // Darkshade Caverns II
        // 931 => Some("".to_string(), // Elden Hollow II
        // 932 => Some("".to_string(), // Crypt of Hearts II
        // 933 => Some("".to_string(), // Wayrest Sewers II
        // 934 => Some("".to_string(), // Fungal Grotto II
        // 935 => Some("".to_string(), // The Banished Cells II
        // 936 => Some("".to_string(), // Spindleclutch II
        // 973 => Some("".to_string(), // Bloodroot Forge
        // 974 => Some("".to_string(), // Falkreath Hold
        // 975 => Some("".to_string(), // Halls of Fabrication
        // 1000 => Some("".to_string(), // Asylum Sanctorium
        // 1009 => Some("".to_string(), // Fang Lair
        // 1010 => Some("".to_string(), // Scalecaller Peak
        1051 => Some("https://images.uesp.net/c/cf/ON-load-Cloudrest.jpg".to_string()), // Cloudrest
        // 1052 => Some("".to_string(), // Moon Hunter Keep
        // 1055 => Some("".to_string(), // March of Sacrifices
        // 1080 => Some("".to_string(), // Frostvault
        // 1081 => Some("".to_string(), // Depths of Malatar
        // 1121 => Some("".to_string(), // Sunspire
        // 1122 => Some("".to_string(), // Moongrave Fane
        // 1123 => Some("".to_string(), // Lair of Maarselok
        // 1152 => Some("".to_string(), // Icereach
        // 1153 => Some("".to_string(), // Unhallowed Grave
        // 1196 => Some("".to_string(), // Kyne's Aegis
        // 1197 => Some("".to_string(), // Stone Garden
        // 1201 => Some("".to_string(), // Castle Thorn
        // 1228 => Some("".to_string(), // Black Drake Villa
        // 1229 => Some("".to_string(), // The Cauldron
        // 1267 => Some("".to_string(), // Red Petal Bastion
        // 1301 => Some("".to_string(), // Coral Aerie
        // 1302 => Some("".to_string(), // Shipwright's Regret
        // 1344 => Some("".to_string(), // Dreadsail Reef
        // 1360 => Some("".to_string(), // Earthen Root Enclave
        // 1361 => Some("".to_string(), // Graven Deep
        // 1389 => Some("".to_string(), // Bal Sunnar
        // 1390 => Some("".to_string(), // Scrivener's Hall
        // 1427 => Some("".to_string(), // Sanity's Edge
        // 1470 => Some("".to_string(), // Oathsworn Pit
        // 1471 => Some("".to_string(), // Bedlam Veil
        // 1478 => Some("".to_string(), // Lucent Citadel
        // 1496 => Some("".to_string(), // Exiled Redoubt
        // 1497 => Some("".to_string(), // Lep Seclusa
        // 1548 => Some("".to_string(), // Ossein Cage
        _ => None
    }
}

#[allow(dead_code)]
fn is_trial(zone_id: u16) -> bool {
    match zone_id {
        636 
        | 638
        | 639
        | 975
        | 1000
        | 1051
        | 1121
        | 1196
        | 1344
        | 1427
        | 1478
        | 1548 => true,
        _ => false,
    }
}