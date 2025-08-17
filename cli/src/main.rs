use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use cli::esologs_convert::{build_master_table, split_and_zip_log_by_fight, ESOLogProcessor};
use cli::log_edit::modify_log_file;
use cli::split_log::split_encounter_file_into_log_files;
use parser::ui::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (file_path, query) = parse_config(&args);

    match query {
        "colours" => {
            print_colour_test();
        }
        "modify" => {
            if let Err(e) = modify_log_file(Path::new(file_path)) {
                eprintln!("Error modifying log file: {}", e);
            }
        }
        "split" => {
            if let Err(e) = split_encounter_file_into_log_files(Path::new(file_path)) {
                eprintln!("Error splitting log file: {}", e);
            }
        }
        "esolog" => {
            let mut eso_log_processor = ESOLogProcessor::new();
            if let Err(e) = eso_log_processor.convert_log_file_to_esolog_format(Path::new(file_path)) {
                eprintln!("Error splitting log file: {}", e);
            }

            if let Ok(file) = File::create("C:/Users/H/Downloads/esolog_output.txt") {
                let mut writer = BufWriter::new(file);
                let master_table = build_master_table(&mut eso_log_processor);
                if let Err(e) = write!(writer, "{master_table}") {
                    eprintln!("Error writing master_table: {}", e);
                }
            } else {
                eprintln!("Error creating output file: esolog_output.txt");
                return;
            }

            println!("master table written");
            if let Ok(file) = File::create("C:/Users/H/Downloads/esolog_output2.txt") {
                let mut writer = BufWriter::new(file);

                for line in &eso_log_processor.eso_logs_log.events {
                    if let Err(e) = writeln!(writer, "{line}") {
                        eprintln!("Error writing events: {}", e);
                        break;
                    }
                }

                if let Err(e) = writer.flush() {
                    eprintln!("Error flushing writer: {}", e);
                }
            } else {
                eprintln!("Error creating output file: esolog_output2.txt");
                return;
            }
        }
        "esologzip" => {
            match split_and_zip_log_by_fight(Path::new(file_path), Path::new("C:/Users/H/Downloads/esologzipoutput/")) {
                Ok(_) => {println!("Done split + zip")},
                Err(e) => println!("{e}"),
            }
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