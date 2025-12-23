use std::ops::Deref;
use std::rc::Rc;

use esologtool_common::{EncounterReportCode, LabelValue, UploadSettings};
use futures::StreamExt;
use tauri_sys::core::{invoke, invoke_result};
use tauri_sys::event;
use web_sys::HtmlInputElement;
use yew::{function_component, html, use_context, Html, use_state, Callback, events::Event, TargetCast};
use yew::prelude::*;
use yew_icons::IconId;
use yew_router::hooks::use_navigator;
use crate::app::{ESOLogUploadSettings, LoginContext};
use crate::routes::Route;
use crate::ui::icon_button::{BackArrow, IconButton};
use crate::ui::style::*;

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
    let tag = use_state(|| None::<i32>);
    let description = use_state(|| String::new());
    let rewind = use_state(|| None::<bool>);
    let description_memory = use_state(|| None::<bool>);
    let report_code = use_state(|| None::<String>);
    let error = use_state(|| None::<String>);
    let is_uploading = use_state(|| UploadState::None);
    let has_been_deleted = use_state(|| false);

    let saved_settings = (*upload_settings_ctx).as_ref().map(|rc| rc.as_ref());
    let selected_guild = saved_settings.map(|s| s.guild).unwrap_or_else(|| -1);
    let selected_region = saved_settings.map(|s| s.region).unwrap_or_else(|| 1);
    let selected_visibility = saved_settings.map(|s| s.visibility).unwrap_or_else(|| 2);
    let selected_rewind = saved_settings.map(|s| s.rewind).unwrap_or_else(|| false);
    let selected_description_remember = saved_settings.map(|s| s.remember_description).unwrap_or_else(|| false);
    let selected_tag = saved_settings.map(|s| s.tag).unwrap_or_else(|| None);

    {
        let description = description.clone();
        let saved_settings = (*upload_settings_ctx).clone();
        let description_memory = description_memory.clone();

        use_effect_with(saved_settings,
            move |saved_settings| {
                if let Some(settings) = saved_settings.as_ref() {
                    if settings.remember_description {
                        description.set(settings.description.clone());
                        description_memory.set(Some(true));
                    }
                }
                || ()
            },
        );
    }
    let active_guild_id: i32 = guild.unwrap_or(selected_guild);
    {
        let tag = tag.clone();
        let login_ctx = login_ctx.clone();
        let selected_tag = selected_tag.clone();

        use_effect_with(active_guild_id, move |guild_id| {
            if let Some(login) = login_ctx.as_ref() {
                if let Some(tags) = login.tags_for_guild(*guild_id) {
                    let selected = selected_tag.unwrap_or(-1);
                    if tags.iter().any(|t| t.value == selected) {
                        tag.set(Some(selected));
                        return;
                    }
                }
            }
            tag.set(None);
        });
    }
    let guild_tags: Option<&Vec<LabelValue>> = login_ctx
        .as_ref()
        .and_then(|login| {
            login.tags_for_guild(active_guild_id)
        });

    let tag_options = if let Some(tags) = guild_tags {
        tags.iter()
            .map(|t| {
                html! {
                    <option value={t.value.to_string()} selected={t.value == selected_tag.unwrap_or(-1)}>
                        { &t.label }
                    </option>
                }
            })
            .collect::<Html>()
    } else {
        html! {}
    };

    let upload_progress = use_state(|| None::<String>);

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

    let upload_effect = upload_progress.clone();
    use_effect(move || {
        let upload_progress = upload_effect.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(mut events) = event::listen::<String>("upload_progress").await {
                while let Some(e) = events.next().await {
                    upload_progress.set(Some(e.payload));
                }
            }
        });
        || ()
    });

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
    let on_tag_change = {
        let tag = tag.clone();
        Callback::from(move |e: Event| {
            let sel: HtmlInputElement = e.target_unchecked_into();
            let parsed = sel.value().parse::<i32>().ok();
            tag.set(parsed);
        })
    };
    let on_description_input = {
        let description = description.clone();
        Callback::from(move |e: InputEvent| {
            let value = e.target_dyn_into::<web_sys::HtmlTextAreaElement>().unwrap().value();
            description.set(value);
        })
    };
    let on_rewind_change = {
        let rewind = rewind.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            rewind.set(Some(input.checked()));
        })
    };
    let on_memory_change = {
        let description_memory = description_memory.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            description_memory.set(Some(input.checked()));
        })
    };

    let upload_log = {
        let upload_progress = upload_progress.clone();
        let report_code = report_code.clone();
        let error = error.clone();
        let is_uploading = is_uploading.clone();
        let guild = guild.clone();
        let region = region.clone();
        let visibility = visibility.clone();
        let description = description.clone();
        let description_memory = description_memory.clone();
        let tag = tag.clone();
        move |_| {
            let upload_progress = upload_progress.clone();
            let report_code = report_code.clone();
            let error = error.clone();
            let is_uploading = is_uploading.clone();
            let guild = guild.clone();
            let region = region.clone();
            let visibility = visibility.clone();
            let description = description.clone();
            let description_memory = description_memory.clone();
            let tag = tag.clone();
            wasm_bindgen_futures::spawn_local(async move {
                invoke::<()>("pick_and_load_file", &()).await;

                is_uploading.set(UploadState::UploadingLog);
                let settings = UploadSettings { 
                    guild: guild.unwrap_or(selected_guild),
                    visibility: visibility.unwrap_or(selected_visibility),
                    region: region.unwrap_or(selected_region),
                    description: description.to_string(),
                    rewind: false,
                    tag: *tag,
                    remember_description: description_memory.unwrap_or(false),
                };
                upload_progress.set(None);
                error.set(None);
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
        let upload_progress = upload_progress.clone();
        let report_code = report_code.clone();
        let error = error.clone();
        let is_uploading = is_uploading.clone();
        let guild = guild.clone();
        let region = region.clone();
        let visibility = visibility.clone();
        let description = description.clone();
        let rewind = rewind.clone();
        let description_memory = description_memory.clone();
        let tag = tag.clone();
        move |_| {
            let upload_progress = upload_progress.clone();
            let report_code = report_code.clone();
            let error = error.clone();
            let is_uploading = is_uploading.clone();
            let guild = guild.clone();
            let region = region.clone();
            let visibility = visibility.clone();
            let description = description.clone();
            let rewind = rewind.clone();
            let description_memory = description_memory.clone();
            let tag = tag.clone();
            wasm_bindgen_futures::spawn_local(async move {
                invoke::<()>("pick_and_load_folder", &()).await;

                is_uploading.set(UploadState::LiveLogging);
                let settings = UploadSettings { 
                    guild: guild.unwrap_or(selected_guild),
                    visibility: visibility.unwrap_or(selected_visibility),
                    region: region.unwrap_or(selected_region),
                    description: description.to_string(),
                    rewind: rewind.unwrap_or(selected_rewind),
                    tag: *tag,
                    remember_description: description_memory.unwrap_or(false),
                };
                upload_progress.set(None);
                error.set(None);
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
        let upload_progress = upload_progress.clone();
        Callback::from(move |_| {
            let is_uploading = is_uploading.clone();
            let upload_progress = upload_progress.clone();
            wasm_bindgen_futures::spawn_local(async move {
                invoke::<()>("cancel_upload_log", &()).await;
                is_uploading.set(UploadState::None);
                upload_progress.set(None);
            });
        })
    };

    let delete_log_file = {
        let error = error.clone();
        let has_been_deleted = has_been_deleted.clone();
        Callback::from(move |_| {
            let error = error.clone();
            let has_been_deleted = has_been_deleted.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match invoke_result::<(), String>("delete_log_file", &()).await {
                    Ok(_) => {has_been_deleted.set(true)},
                    Err(e) => {error.set(Some(e.to_string()))}
                };
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
                <option value={v.value.to_string()} selected={v.value as u8 == selected_visibility}>{ &v.label }</option>
            }
        }).collect::<Html>();

        (guild_options, region_options, visibility_options)
    } else {
        (html! {}, html! {}, html! {})
    };
    let navigator = use_navigator().unwrap();

    {
        let navigator = navigator.clone();
        let has_been_deleted = has_been_deleted.clone();
        use_effect(move || {
            if *has_been_deleted {
                navigator.push(&Route::Home);
            }
            || ()
        });
    }

    html! {
        <div>
            if *is_uploading == UploadState::None {<BackArrow/>}
            <div style="gap: 0px;" class={container_style().clone()}>
                if *is_uploading == UploadState::UploadingLog {
                    <h3>{"Please be patient while your log file is processed. This can take multiple minutes."}</h3>
                    if let Some(progress) = upload_progress.as_ref() {
                        <div>
                            { progress }
                        </div>
                    }
                    <IconButton
                        icon_id={IconId::BootstrapXLg}
                        description={"Cancel upload"}
                        onclick={Some(cancel_upload.clone())}
                        class={icon_border_style().clone()}
                        width={"2em"}
                        height={"2em"}
                    />
                    if let Some(code) = report_code.clone().deref() {
                        <a class={text_link_style().clone()} style={"font-size: large;margin-top:1em;margin-bottom:1em;"} href={format!("https://www.esologs.com/reports/{code}")} target="_blank" rel="noopener noreferrer">
                            {"Click to open your encounter log"}
                        </a>
                    }
                } else if *is_uploading == UploadState::LiveLogging { 
                    <h3 style="margin-top:2em;">{"You are now live logging! Press the stop button once everything has been uploaded."}</h3>
                    if let Some(progress) = upload_progress.as_ref() {
                        <div>
                            { progress }
                        </div>
                    } 
                    <IconButton
                        icon_id={IconId::BootstrapXLg}
                        description={"Stop live log"}
                        onclick={Some(cancel_upload.clone())}
                        class={icon_border_style().clone()}
                        width={"2em"}
                        height={"2em"}
                    />
                    if let Some(code) = report_code.clone().deref() {
                        <a class={text_link_style().clone()} style={"font-size: large;margin-top:1em;margin-bottom:1em;"} href={format!("https://www.esologs.com/reports/{code}")} target="_blank" rel="noopener noreferrer">
                            {"Click to open your live log"}
                        </a>
                    }
                }
                if let Some(_) = report_code.clone().deref() {
                    if !*has_been_deleted && *is_uploading == UploadState::None {
                        if let Some(code) = report_code.clone().deref() {
                            <a class={text_link_style().clone()} style={"font-size: large;margin-top:1em;margin-bottom:1em;"} href={format!("https://www.esologs.com/reports/{code}")} target="_blank" rel="noopener noreferrer">
                                {"Click to open your encounter log"}
                            </a>
                        }
                        <IconButton
                            icon_id={IconId::BootstrapTrash3}
                            description={"Delete uploaded file permanently"}
                            onclick={Some(delete_log_file.clone())}
                            class={icon_border_style().clone()}
                            width={"2em"}
                            height={"2em"}
                        /> 
                    }
                } else if *is_uploading == UploadState::None {
                    <div style="width: min-content; margin: 0.5em;">
                        <h3>{"Specify how to upload your log:"}</h3>
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
                            { if guild_tags.is_some() {
                                html! {
                                    <select onchange={on_tag_change}>
                                        { tag_options }
                                    </select>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                        <h3 style="margin-top:1em;">{"Give your log a description:"}</h3>
                        <textarea
                            name={"description"}
                            autocomplete="off"
                            value={(*description).clone()}
                            oninput={on_description_input}
                            placeholder="Description"
                            style="width:100%;padding:0.2em;border:0px;resize:none;margin-bottom:1.5em;"
                        />
                        <div style="margin-top:1em;">
                            <h3 style="display:inline; margin-right:1em;">
                                {"(Live log) Upload entire file:"}
                            </h3>
                            <input 
                                type="checkbox"
                                checked={rewind.unwrap_or(selected_rewind)}
                                onchange={on_rewind_change}
                            />
                        </div>

                        <div style="margin-top:1em; margin-bottom:1.5em;">
                            <h3 style="display:inline; margin-right:1em;">
                                {"Remember Description:"}
                            </h3>
                            <input 
                                type="checkbox"
                                checked={description_memory.unwrap_or(selected_description_remember)}
                                onchange={on_memory_change}
                            />
                        </div>
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
                }
                if let Some(err) = &*error {
                    <div style="color: red; margin-bottom: 1em;">{ err }</div>
                }
            </div>
        </div>
    }
}