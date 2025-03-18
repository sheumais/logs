mod unit;
mod fight;
mod player;
mod effect;
mod event;
mod log;
mod ui;
mod set;

use std::io::{self};
use player::{GearPiece, Player};
use yew::prelude::*;
use crate::log::Log;
use web_sys::{wasm_bindgen::{prelude::Closure, JsCast}, File, FileReader};

#[function_component]
fn App() -> Html {
    let file_content = use_state(|| String::new());
    let logs = use_state(|| Vec::<Log>::new());

    let on_file_change = {
        let file_content = file_content.clone();
        let logs = logs.clone();
        Callback::from(move |event: Event| {
            let input: web_sys::HtmlInputElement = event.target_unchecked_into();
            if let Some(file) = input.files().and_then(|files| files.get(0)) {
                parse_file(file, file_content.clone(), logs.clone());
            }
        })
    };

    let all_players: Vec<Player> = logs.iter().flat_map(|log| {
        log.fights.iter().flat_map(|fight| {
            fight.players.iter()
        })
    }).cloned().collect();

    fn render_gear_piece_data(gear: &GearPiece) -> Html {
        let level = player::veteran_level_to_cp(gear.level, gear.is_cp);
        let cp_string = if gear.is_cp {"CP"} else {"Level "};
        let set_name = set::get_set_name(gear.set_id);
        html! {
            <>
                <td>{ if set::is_weapon_slot(&gear.slot) {
                let weapon_type = set::get_weapon_type_from_hashmap(gear.item_id);
                let weapon_type_name = set::get_weapon_name(weapon_type);
                format!("{}", weapon_type_name)} else {format!("{:?}", gear.slot)} }</td>
                <td>{ format!("{:?}", gear.quality)}</td>
                <td>{ format!("{}{:?}", cp_string, level)}</td>
                <td>{ format!("{}", set_name.unwrap_or("Unknown")) }</td>
            </>
        }
    }

    html! {
        <div>
        <div id="plot-div"></div>
            <h1>{ "Upload File" }</h1>
            <input type="file" onchange={on_file_change} />
            <p>{ "File Content: " }</p>
            <h1>{ "Player Loadout Information" }</h1>
            { for all_players.iter().map(|player| 
                if player.gear != player::empty_loadout() {html! {
                    <div>
                        <h2>{ format!("Loadout for {}", player.display_name) }</h2>
                        <table border="1">
                            <thead>
                                <tr>
                                    <th>{ "Gear Slot" }</th>
                                    <th>{ "Item Quality" }</th>
                                    <th>{ "Level / CP" }</th>
                                    <th>{ "Set Name" }</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for [
                                    &player.gear.head,
                                    &player.gear.shoulders,
                                    &player.gear.chest,
                                    &player.gear.hands,
                                    &player.gear.waist,
                                    &player.gear.legs,
                                    &player.gear.feet,
                                    &player.gear.neck,
                                    &player.gear.ring1,
                                    &player.gear.ring2,
                                    &player.gear.main_hand,
                                    &player.gear.off_hand,
                                    &player.gear.main_hand_backup,
                                    &player.gear.off_hand_backup
                                ].iter().map(|gear_piece| 
                                    if *gear_piece != &player::empty_gear_piece() {
                                        html! {<tr>{ render_gear_piece_data(gear_piece) }</tr>}
                                    } else {
                                        html! {<div></div>}
                                    }
                                )}
                            </tbody>
                        </table>
                    </div>
                }} else {html!(<div></div>)}) }
        </div>
    }
}

fn parse_file(file: File, file_content: UseStateHandle<String>, logs: UseStateHandle<Vec<Log>>) {
    let reader = FileReader::new().unwrap();
    let reader_clone = reader.clone();

    let file_content = file_content.clone();
    let logs = logs.clone();

    let onloadend = Closure::wrap(Box::new(move |_: web_sys::ProgressEvent| {
        if let Some(result) = reader_clone.result().unwrap().as_string() {
            file_content.set(result.clone());
            if let Ok(parsed_logs) = parse_logs_from_text(&result) {
                logs.set(parsed_logs);
            }
        }
    }) as Box<dyn FnMut(web_sys::ProgressEvent)>);

    reader.set_onloadend(Some(onloadend.as_ref().unchecked_ref()));

    reader.read_as_text(&file).unwrap();

    onloadend.forget();
}

fn parse_logs_from_text(content: &str) -> io::Result<Vec<Log>> {
    let mut logs = Vec::new();
    let mut current_log = Log::new();

    for line in content.lines() {
        let mut in_brackets = false;
        let mut current_segment_start = 0;
        let mut parts = Vec::new();

        for (i, char) in line.char_indices() {
            match char {
                '[' => {
                    in_brackets = true;
                    current_segment_start = i + 1;
                }
                ']' => {
                    in_brackets = false;
                    parts.push(&line[current_segment_start..i]);
                    current_segment_start = i + 1;
                }
                ',' if !in_brackets => {
                    parts.push(&line[current_segment_start..i]);
                    current_segment_start = i + 1;
                }
                _ => {}
            }
        }

        if current_segment_start < line.len() {
            parts.push(&line[current_segment_start..]);
        }
        parts.retain(|part| !part.is_empty());
        let second_value = parts.get(1).unwrap_or(&"");

        match *second_value {
            "BEGIN_LOG" => {
                if !current_log.is_empty() {
                    logs.push(current_log);
                }
                current_log = Log::new();
                current_log.parse_line(parts);
            }
            "END_LOG" => {
                logs.push(current_log);
                current_log = Log::new();
            }
            _ => {
                current_log.parse_line(parts);
            }
        }
    }

    if !current_log.is_empty() {
        logs.push(current_log);
    }

    Ok(logs)
}

fn main() {
    yew::Renderer::<App>::new().render();
}