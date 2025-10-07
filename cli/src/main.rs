use std::collections::{HashMap, HashSet};
use std::{env, fs};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use cli::esologs_convert::{build_master_table, split_and_zip_log_by_fight, ESOLogProcessor};
use cli::esologs_format::{ESOLogsEvent, ESOLogsLineType};
use cli::log_edit::modify_log_file;
use cli::split_log::split_encounter_file_into_log_files;
use ftail::Ftail;
use log::LevelFilter;
use parser::ui::*;

fn main() {
    let result = Ftail::new()
        .console(LevelFilter::Trace)
        .init();
    match result {
        Ok(_) => log::info!("Logging initialised"),
        Err(e) => println!("Error initialising logging: {}", e),
    }

    let args: Vec<String> = env::args().collect();
    let (file_path, query) = parse_config(&args);

    match query {
        "colours" => {
            print_colour_test();
        }
        "modify" => {
            if let Err(e) = modify_log_file(Path::new(file_path)) {
                log::error!("Error modifying log file: {}", e);
            }
        }
        "split" => {
            if let Err(e) = split_encounter_file_into_log_files(Path::new(file_path)) {
                log::error!("Error splitting log file: {}", e);
            }
        }
        "esolog" => {
            let mut eso_log_processor = ESOLogProcessor::new();
            if let Err(e) = eso_log_processor.convert_log_file_to_esolog_format(Path::new(file_path)) {
                log::error!("Error splitting log file: {}", e);
            }

            if let Ok(file) = File::create("C:/Users/H/Downloads/master_table.txt") {
                let mut writer = BufWriter::new(file);
                let master_table = build_master_table(&mut eso_log_processor);
                if let Err(e) = write!(writer, "{master_table}") {
                    log::error!("Error writing master_table: {}", e);
                }
            } else {
                log::error!("Error creating output file: master_table.txt");
                return;
            }

            log::info!("master table written");
            if let Ok(file) = File::create("C:/Users/H/Downloads/report_segments.txt") {
                let mut writer = BufWriter::new(file);

                for line in &eso_log_processor.eso_logs_log.events {
                    if let Err(e) = writeln!(writer, "{line}") {
                        log::warn!("Error writing events: {}", e);
                        break;
                    }
                }

                if let Err(e) = writer.flush() {
                    log::warn!("Error flushing writer: {}", e);
                }
            } else {
                log::error!("Error creating output file: report_segments.txt");
                return;
            }
        }
        "esologzip" => {
            let noop = |_progress: u8| {};
            let dummy_cancel = std::sync::atomic::AtomicBool::new(false);
            split_and_zip_log_by_fight(file_path, r#"C:\Users\H\AppData\Local\Temp\esologtool_temporary"#, noop, &dummy_cancel).expect("esologzip shouldn't error");
        }
        "aoe" => {
            let mut eso_log_processor = ESOLogProcessor::new();

            if let Err(e) = eso_log_processor.convert_log_file_to_esolog_format(Path::new(file_path)) {
                log::error!("Error converting log file: {}", e);
                return;
            }

            log::info!("Finished parsing lines, now looking for aoe abilities...");

            let mut already_found: HashSet<u32> = HashSet::new();
            let output_file = "C:/Users/H/Downloads/aoe.csv";
            if let Ok(file) = File::open(output_file) {
                let reader = BufReader::new(file);
                for line in reader.lines().flatten() {
                    if let Some((id_str, _name)) = line.split_once(',') {
                        if let Ok(id) = id_str.parse::<u32>() {
                            already_found.insert(id);
                        }
                    }
                }
            }

            let non_aoe_ids: HashSet<u32> = vec![
                41839,
                41838,
                243742,
                190179,
                98438,
                187843,
                61945,
                17895,
                220863,
                183430,
                107203,
                93307,
                147743,
                79707,
                79025,
                17899,
                46743,
                17902,
                18084,21929,148801,21925,215779,148797,148800,21487,21481 // status effects
            ].into_iter().collect();

            let mut aoe_candidates: HashMap<u32, Vec<(u64, usize, usize)>> = HashMap::new();

            for event in &eso_log_processor.eso_logs_log.events {
                if let ESOLogsEvent::CastLine(cast_line) = event {
                    if matches!(cast_line.line_type, ESOLogsLineType::Damage | ESOLogsLineType::DotTick) {
                        if cast_line.cast.source_allegiance == 16 && cast_line.cast.target_allegiance == 64 {
                            if let Some(cast_info) = &cast_line.cast_information {
                                if cast_info.hit_value > 0 {
                                    let cast_id = cast_line.cast.cast_id_origin;
                                    let ts = cast_line.timestamp;
                                    let target = cast_line.buff_event.target_unit_index;
                                    let buff_index = cast_line.buff_event.buff_index;
                                    aoe_candidates.entry(cast_id).or_default().push((ts, target, buff_index));
                                }
                            }
                        }
                    }
                }
            }

            let mut aoe_buff_indexes: HashSet<usize> = HashSet::new();

            for (_cast_id, hits) in aoe_candidates {
                let mut hits_sorted = hits.clone();
                hits_sorted.sort_by_key(|(ts, _, _)| *ts);

                'outer: for i in 0..hits_sorted.len() {
                    let (ts_i, target_i, buff_idx_i) = hits_sorted[i];
                    let mut unique_targets = HashSet::new();
                    unique_targets.insert(target_i);

                    for j in i + 1..hits_sorted.len() {
                        let (ts_j, target_j, buff_idx_j) = hits_sorted[j];
                        if ts_j.saturating_sub(ts_i) <= 2 {
                            unique_targets.insert(target_j);
                            if unique_targets.len() > 1 {
                                aoe_buff_indexes.insert(buff_idx_i);
                                aoe_buff_indexes.insert(buff_idx_j);
                                break 'outer;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(output_file)
                .expect("Unable to open or create output file");

            for idx in aoe_buff_indexes {
                if let Some(buff) = eso_log_processor.eso_logs_log.buffs.get(idx) {
                    if non_aoe_ids.contains(&buff.id) || already_found.contains(&buff.id) {
                        continue;
                    }
                    log::info!("{},{}", buff.id, buff.name);
                    writeln!(file, "{},{}", buff.id, buff.name)
                        .expect("Unable to write to file");
                    already_found.insert(buff.id);
                }
            }
        }
        "aoesql" => {
            let file_path = "C:/Users/H/Downloads/aoe.csv";
            let content = fs::read_to_string(file_path).expect("should always be able to read aoe.csv");
            let ids: Vec<&str> = content
                .lines()
                .filter_map(|line| line.split(',').next())
                .collect();
            let id_list = ids.join(",");
            let sql = format!("ability.id IN ({})", id_list);
            log::info!("{}", sql);
        }
        "parentzones" => {
            parser::zone::print_parent_zones();
        }
        "dungeons" => {
            parser::zone::print_dungeon_zones();
        }
        _ => {}
    }
}

fn parse_config(args: &[String]) -> (&str, &str) {
    let mut query = "";
    let file_path = &args[1];
    if args.len() > 2 {
        query = &args[2];
    }

    (file_path, query)
}