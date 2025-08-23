use std::rc::Rc;

use cli::esologs_format::{EncounterReportCode, UploadSettings};
use futures::StreamExt;
use tauri_sys::core::{invoke, invoke_result};
use tauri_sys::event;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures;
use web_sys::HtmlInputElement;
use yew::{function_component, html, use_context, Html, use_state, Callback, events::Event, TargetCast, use_effect_with};
use yew::prelude::*;
use crate::app::{ESOLogUploadSettings, LoginContext};
use crate::ui::icon_button::{BackArrow, IconButton};
use crate::ui::style::*;
use yew_icons::IconId;
use std::ops::Deref;

#[derive(PartialEq)]
pub enum UploadState {
    None,
    UploadingLog,
    LiveLogging,
}

#[function_component(UploadScreen)]
pub fn upload() -> Html {
    let login_ctx = use_context::<LoginContext>().expect("LoginContext should always be an Option");
    let upload_settings_ctx = use_context::<ESOLogUploadSettings>().expect("Upload Settings should always be an Option");

    {
        let upload_settings_ctx = upload_settings_ctx.clone();
        use_effect(move || {
            wasm_bindgen_futures::spawn_local(async move {
                match invoke::<Option<UploadSettings>>(
                    "get_saved_upload_settings",
                    &serde_json::json!({})
                ).await {
                    Some(settings) => upload_settings_ctx.set(Some(Rc::new(settings))),
                    None => upload_settings_ctx.set(None),
                }
            });
            || ()
        });
    }

    let guild = use_state(|| None::<i32>);
    let region = use_state(|| None::<u8>);
    let visibility = use_state(|| None::<u8>);
    let description = use_state(|| "".to_string());
    let report_code = use_state(|| None::<String>);
    let error = use_state(|| None::<String>);
    let is_uploading = use_state(|| UploadState::None);
    let has_been_deleted = use_state(|| false);
    // Remove state management - we'll use raw DOM manipulation instead

    let saved_settings = (*upload_settings_ctx).as_ref().map(|rc| rc.as_ref());
    let selected_guild = saved_settings.map(|s| s.guild).unwrap_or_else(|| -1);
    let selected_region = saved_settings.map(|s| s.region).unwrap_or_else(|| 1);
    let selected_visibility = saved_settings.map(|s| s.visibility).unwrap_or_else(|| 2);

    let report_effect = report_code.clone();
    use_effect(move || {
        let code = report_effect.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(mut events) = event::listen::<String>("live_log_code").await {
                while let Some(e) = events.next().await {
                    code.set(Some(e.payload));
                }
            }
        });
        || ()
    });

    // Raw JavaScript DOM manipulation for status messages - ensure single listener
    use_effect_with((), move |_| {
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Spawn the event listener task
        let handle = wasm_bindgen_futures::spawn_local(async move {
            if let Ok(mut events) = event::listen::<String>("upload_status").await {
                while let Some(e) = events.next().await {
                    let new_message = e.payload;
                    web_sys::console::log_1(&format!("Received status message: {}", new_message).into());
                    
                    // Try both console elements - append to whichever one exists and is visible
                    let console_ids = ["upload-console", "live-console"];
                    
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            for console_id in &console_ids {
                                if let Some(console_element) = document.get_element_by_id(console_id) {
                                    // Check if this console element is currently visible
                                    if let Ok(html_element) = console_element.clone().dyn_into::<web_sys::HtmlElement>() {
                                        if html_element.offset_parent().is_some() {
                                            // Create new div element for this message
                                            if let Ok(div) = document.create_element("div") {
                                                div.set_text_content(Some(&new_message));
                                                div.set_attribute("style", "margin: 2px 0;").ok();
                                                
                                                // Append the new div
                                                console_element.append_child(&div).ok();
                                                
                                                // Auto-scroll to bottom
                                                html_element.set_scroll_top(html_element.scroll_height());
                                            }
                                            break; // Only append to the first visible console
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Check if we should stop (cleanup signal)
                    if let Ok(_) = rx.try_recv() {
                        break;
                    }
                }
            }
        });
        
        // Return cleanup function
        move || {
            tx.send(()).ok(); // Signal to stop the listener
        }
    });

    // Scroll effect removed - now handled directly in event handler


    let on_guild_change = {
        let guild = guild.clone();
        Callback::from(move |e: Event| {
            let sel: HtmlInputElement = e.target_unchecked_into();
            let parsed = sel.value().parse::<i32>().ok();
            guild.set(parsed);
        })
    };
    let on_region_change = {
        let region = region.clone();
        Callback::from(move |e: Event| {
            let sel: HtmlInputElement = e.target_unchecked_into();
            let parsed = sel.value().parse::<u8>().ok();
            region.set(parsed);
        })
    };
    let on_visibility_change = {
        let visibility = visibility.clone();
        Callback::from(move |e: Event| {
            let sel: HtmlInputElement = e.target_unchecked_into();
            let parsed = sel.value().parse::<u8>().ok();
            visibility.set(parsed);
        })
    };
    let on_description_input = {
        let description = description.clone();
        Callback::from(move |e: InputEvent| {
            let value = e.target_dyn_into::<web_sys::HtmlTextAreaElement>().unwrap().value();
            description.set(value);
        })
    };

    let upload_log = {
        let report_code = report_code.clone();
        let error = error.clone();
        let is_uploading = is_uploading.clone();
        let guild = guild.clone();
        let region = region.clone();
        let visibility = visibility.clone();
        let description = description.clone();
        move |_| {
            let report_code = report_code.clone();
            let error = error.clone();
            let is_uploading = is_uploading.clone();
            let guild = guild.clone();
            let region = region.clone();
            let visibility = visibility.clone();
            let description = description.clone();
                wasm_bindgen_futures::spawn_local(async move {
                invoke::<()>("pick_and_load_file", &()).await;

                // Clear previous messages when starting new upload
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Some(console_element) = document.get_element_by_id("upload-console") {
                            console_element.set_inner_html("");
                        }
                    }
                }
                is_uploading.set(UploadState::UploadingLog);
                let settings = UploadSettings { 
                    guild: guild.unwrap_or(-1), 
                    visibility: visibility.unwrap_or(2), 
                    region: region.unwrap_or(1),
                    description: description.to_string(),
                };
                match invoke_result::<EncounterReportCode, String>("upload_log",  
                    &serde_json::json!({
                        "uploadSettings": settings,
                    })).await {
                        Ok(code) => {report_code.set(Some(code.code))}
                        Err(err) => {error.set(Some(err.to_string()))}
                    };
                is_uploading.set(UploadState::None);
            });
        }
    };

    let live_log = {
        let report_code = report_code.clone();
        let error = error.clone();
        let is_uploading = is_uploading.clone();
        let guild = guild.clone();
        let region = region.clone();
        let visibility = visibility.clone();
        let description = description.clone();
        move |_| {
            let report_code = report_code.clone();
            let error = error.clone();
            let is_uploading = is_uploading.clone();
            let guild = guild.clone();
            let region = region.clone();
            let visibility = visibility.clone();
            let description = description.clone();
                wasm_bindgen_futures::spawn_local(async move {
                invoke::<()>("pick_and_load_folder", &()).await;

                // Clear previous messages when starting new live log
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Some(console_element) = document.get_element_by_id("live-console") {
                            console_element.set_inner_html("");
                        }
                    }
                }
                is_uploading.set(UploadState::LiveLogging);
                let settings = UploadSettings { 
                    guild: guild.unwrap_or(-1), 
                    visibility: visibility.unwrap_or(2), 
                    region: region.unwrap_or(1),
                    description: description.to_string(),
                };
                match invoke_result::<EncounterReportCode, String>("live_log_upload",  
                    &serde_json::json!({
                        "uploadSettings": settings,
                    })).await {
                        Ok(code) => {report_code.set(Some(code.code))}
                        Err(err) => {error.set(Some(err.to_string()))}
                    };
                is_uploading.set(UploadState::None);
            });
        }
    };

    let cancel_upload = {
        let is_uploading = is_uploading.clone();
        Callback::from(move |_| {
            let is_uploading = is_uploading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                invoke::<()>("cancel_upload_log", &()).await;
                is_uploading.set(UploadState::None);
            });
        })
    };

    let delete_log_file = {
        let has_been_deleted = has_been_deleted.clone();
        Callback::from(move |_| {
            let has_been_deleted = has_been_deleted.clone();
            wasm_bindgen_futures::spawn_local(async move {
                invoke::<()>("delete_log_file", &()).await;
                has_been_deleted.set(true)
            });
        })
    };

    let (guild_options, region_options, visibility_options) = if let Some(login) = &*login_ctx {
        let guild_options = login.guild_select_items.iter().map(|g| {
            html! {
                <option value={g.value.to_string()} selected={g.value == selected_guild}>{ &g.label }</option>
            }
        }).collect::<Html>();

        let region_options = login.region_or_server_select_items.iter().map(|r| {
            html! {
                <option value={r.value.to_string()} selected={r.value == selected_region}>{ &r.label }</option>
            }
        }).collect::<Html>();

        let visibility_options = login.report_visibility_select_items.iter().map(|v| {
            html! {
                <option value={v.value.to_string()} selected={v.value == selected_visibility}>{ &v.label }</option>
            }
        }).collect::<Html>();

        (guild_options, region_options, visibility_options)
    } else {
        (html! {}, html! {}, html! {})
    };

    html! {
        <div>
            if *is_uploading == UploadState::None {
<BackArrow/>
            }
            <div class={container_style().clone()}>
                if *is_uploading == UploadState::UploadingLog {
                    <h3 style="margin-top:2em;">{"Please be patient while your log file is processed. This can take minutes."}</h3>
                    <div 
                        style="
                            margin: 1em 0;
                            padding: 0.5em;
                            background-color: #000;
                            border: 1px solid #333;
                            border-radius: 4px;
                            color: #0f0;
                            font-family: 'Courier New', Monaco, monospace;
                            font-size: 12px;
                            height: 200px;
                            overflow-y: auto;
                            white-space: pre-wrap;
                            word-break: break-word;
                            line-height: 1.2;
                            text-align: left;
                            width: 90%;
                            max-width: 90%;
                            box-sizing: border-box;
                        "
                        id="upload-console"
                    >
                        // Messages will be appended here via raw DOM manipulation
                    </div>
                    <IconButton
                        icon_id={IconId::BootstrapXLg}
                        description={"Cancel upload"}
                        onclick={Some(cancel_upload.clone())}
                        class={icon_border_style().clone()}
                        width={"2em"}
                        height={"2em"}
                    />
                } else if *is_uploading == UploadState::LiveLogging { 
                    <h3 style="margin-top:2em;">{"You are now live logging! Press the stop button once everything has been uploaded."}</h3>
                    <div 
                        style="
                            margin: 1em 0;
                            padding: 0.5em;
                            background-color: #000;
                            border: 1px solid #333;
                            border-radius: 4px;
                            color: #0f0;
                            font-family: 'Courier New', Monaco, monospace;
                            font-size: 12px;
                            height: 200px;
                            overflow-y: auto;
                            white-space: pre-wrap;
                            word-break: break-word;
                            line-height: 1.2;
                            text-align: left;
                            width: 90%;
                            max-width: 90%;
                            box-sizing: border-box;
                        "
                        id="live-console"
                    >
                        // Messages will be appended here via raw DOM manipulation
                    </div>
                    if let Some(code) = report_code.clone().deref() {
                        <a class={text_link_style().clone()} style={"font-size: large;margin-top:1em;margin-bottom:1em;"} href={format!("https://www.esologs.com/reports/{}", code)} target="_blank" rel="noopener noreferrer">
                            {"Click to open your live log"}
                        </a>
                    }
                    <IconButton
                        icon_id={IconId::BootstrapXLg}
                        description={"Stop live log"}
                        onclick={Some(cancel_upload.clone())}
                        class={icon_border_style().clone()}
                        width={"2em"}
                        height={"2em"}
                    />
                } else if report_code.is_some() {
                    if let Some(code) = report_code.clone().deref() {
                        <a class={text_link_style().clone()} style={"font-size: large;margin-top:5em;margin-bottom:3em;"} href={format!("https://www.esologs.com/reports/{}", code)} target="_blank" rel="noopener noreferrer">
                            {"Click to open your encounter log"}
                        </a>
                        if !*has_been_deleted {
                            <IconButton
                                icon_id={IconId::BootstrapTrash3}
                                description={"Delete uploaded file permanently"}
                                onclick={Some(delete_log_file.clone())}
                                class={icon_border_style().clone()}
                                width={"2em"}
                                height={"2em"}
                            /> 
                        }
                       
                    }
                } else {
                    <div style="width: min-content; margin: 0.5em;">
                        <h3 style="margin-top:2em;">{"Specify how to upload your log:"}</h3>
                        <div style="display:inline-flex;gap:1em;">
                            <select onchange={on_guild_change} name={"guild"} autocomplete="off">
                                { guild_options }
                            </select>
                            <select onchange={on_region_change} name={"region"} autocomplete="off">
                                { region_options }
                            </select>
                            <select onchange={on_visibility_change} name={"visibility"} autocomplete="off">
                                { visibility_options }
                            </select>
                        </div>
                        <h3 style="margin-top:2em;">{"Give your log a description:"}</h3>
                        <textarea
                            name={"description"}
                            autocomplete="off"
                            value={(*description).clone()}
                            oninput={on_description_input}
                            placeholder="Description"
                            style="width:100%;padding:0.2em;border:0px;resize:none;"
                        />
                    </div>
                    <div class={icon_wrapper_style().clone()}>
                        <IconButton
                            icon_id={IconId::LucideUpload}
                            description={"Upload log"}
                            onclick={Some(upload_log.clone())}
                            class={icon_style().clone()}
                        />
                        <IconButton
                            icon_id={IconId::BootstrapFileEarmarkPlay}
                            description={"Start live logging"}
                            onclick={Some(live_log.clone())}
                            class={icon_style().clone()}
                        />
                    </div>
                    if let Some(err) = &*error {
                        <div style="color: red; margin-bottom: 1em;">{ err }</div>
                    }
                }
            </div>
        </div>
    }
}