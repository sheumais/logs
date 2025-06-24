use futures::StreamExt;
use tauri_sys::{core::invoke, event};
use yew::{classes, function_component, html, use_effect, use_state, Callback, Html, UseStateHandle};
use yew_icons::{Icon, IconId};
use yew_router::hooks::use_navigator;
use crate::ui::style::*;
use crate::routes::Route;

struct LiveLogState {
    hovered: UseStateHandle<Option<usize>>,
    live_logging: UseStateHandle<bool>,
    progress: UseStateHandle<u32>,
}

#[function_component(LiveLog)]
pub fn live_log() -> Html {
    let navigator = use_navigator().unwrap();
    let hovered = use_state(|| None::<usize>);
    let live_logging = use_state(|| false);
    let progress = use_state(|| 0);
    let state = LiveLogState {
        hovered,
        live_logging,
        progress,
    };

    let go_home = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::Home))
    };

    let live_log = {
        let state = state.live_logging.clone();
        Callback::from(move |_| {
            let state = state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                invoke::<()>("pick_and_load_folder", &()).await;
                state.set(true);
                invoke::<()>("live_log_from_folder", &()).await;
            });
        })
    };


    {
        let progress = state.progress.clone();
        use_effect(move || {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(mut events) = event::listen::<u32>("live_log_progress").await {
                    while let Some(e) = events.next().await {
                        progress.set(e.payload);
                    }
                }
            });
            || ()
        });
    }

    let buttons = vec![
        (IconId::BootstrapFolderSymlink, icon_border_style().clone(), Some(live_log.clone()), "Live log"),
    ];

    html! {
        <>
            <Icon
                class={back_arrow_style().clone()}
                icon_id={IconId::LucideArrowLeftCircle}
                onclick={go_home}
            />
            <div class={classes!(container_style().clone())}>
                <div class={classes!(
                    if *state.live_logging { none_style().clone() } else { hide_style().clone() }
                )}>
                    {format!("{} new lines written", *state.progress)}
                </div>
                <div class={icon_wrapper_style().clone()}>
                    {
                        for buttons.iter().enumerate().map(|(i, (icon_id, style, onclick, desc))| {
                            let hovered = state.hovered.clone();
                            let onmouseover = {
                                let hovered = hovered.clone();
                                Callback::from(move |_| hovered.set(Some(i)))
                            };
                            let onmouseout = {
                                let hovered = hovered.clone();
                                Callback::from(move |_| hovered.set(None))
                            };
                            html! {
                                <div class={classes!(style.clone(), "icon-button")}
                                     {onmouseover}
                                     {onmouseout}
                                >
                                    <Icon
                                        width={"5em".to_owned()}
                                        height={"5em".to_owned()}
                                        icon_id={icon_id.clone()}
                                        onclick={onclick.clone()}
                                    />
                                    <div class={
                                        if hovered.as_ref() == Some(&i) {icon_description_visible().clone()} else {icon_description().clone()}
                                    }>
                                        {desc}
                                    </div>
                                </div>
                            }
                        })
                    }
                </div>
                <div class={paragraph_style().clone()}>
                    <div class={paragraph_style().clone()}>{"This will create the logs/LogToolLive subfolder in the selected destination. After creation, the contents of Encounter.log will be copied into it with modifications periodically."}</div>
                    <div class={paragraph_style().clone()}>{"To live log with the esologs.com uploader, select the LogToolLive subfolder as the live-logging folder on the uploader."}</div>
                    <div class={paragraph_style().clone()}>{"Do not store anything important in the LogToolLive subfolder. It is deleted regularly."}</div>
                </div>
            </div>
        </>
    }
}