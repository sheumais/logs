use futures::StreamExt;
use tauri_sys::{core::invoke, event};
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_icons::{Icon, IconId};

use crate::{routes::Route, ui::style::{back_arrow_style, container_style, hide_style, icon_description, icon_description_visible, icon_style, icon_wrapper_style, none_style}};

struct SplitCombineState {
    hovered: UseStateHandle<Option<usize>>,
    is_splitting: UseStateHandle<bool>,
    is_combining: UseStateHandle<bool>,
    progress: UseStateHandle<u32>,
}

#[function_component(SplitCombineScreen)]
pub fn split_combine_screen() -> Html {
    let navigator = use_navigator().unwrap();
    let hovered = use_state(|| None::<usize>);
    let is_splitting = use_state(|| false);
    let is_combining = use_state(|| false);
    let progress = use_state(|| 0);
    let state = SplitCombineState {
        hovered,
        is_splitting,
        is_combining,
        progress,
    };

    let go_home = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::Home))
    };

    let split_log = {
        let state = state.is_splitting.clone();
        Callback::from(move |_| {
            let state = state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                invoke::<()>("pick_and_load_file", &()).await;
                state.set(true);
                invoke::<()>("split_encounter_file_into_log_files", &()).await;
            });
        })
    };

    let combine_log = {
        let state = state.is_combining.clone();
        Callback::from(move |_| {
            let state = state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                invoke::<()>("pick_and_load_files", &()).await;
                state.set(true);
                invoke::<()>("combine_encounter_log_files", &()).await;
            });
        })
    };

    {
        let progress = state.progress.clone();
        let is_splitting = state.is_splitting.clone();
        use_effect(move || {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(mut events) = event::listen::<u32>("log_split_progress").await {
                    while let Some(e) = events.next().await {
                        progress.set(e.payload);
                        if e.payload >= 100 {
                            is_splitting.set(false);
                        }
                    }
                }
            });
            || ()
        });
    }

    {
        let progress = state.progress.clone();
        let state = state.is_combining.clone();
        use_effect(move || {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(mut events) = event::listen::<u32>("log_combine_progress").await {
                    while let Some(e) = events.next().await {
                        progress.set(e.payload);
                        if e.payload >= 100 {
                            state.set(false);
                        }
                    }
                }
            });
            || ()
        });
    }

    let buttons = vec![
        (IconId::BootstrapFileEarmarkArrowUp, icon_style().clone(), Some(split_log.clone()), "Split log"),
        (IconId::BootstrapFolder, icon_style().clone(), Some(combine_log.clone()), "Combine logs"),
    ];

    html! {
        <>
            <Icon
                class={classes!(
                    if *state.is_combining || *state.is_splitting { hide_style().clone() } else { none_style().clone() },
                    back_arrow_style()
                )}
                icon_id={IconId::LucideArrowLeftCircle}
                onclick={go_home}
            />
            <div class={container_style().clone()}>
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
                                        class={style.clone()}
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
                <div class={classes!(
                    if *state.is_combining || *state.is_splitting { none_style().clone() } else { hide_style().clone() }
                )}>
                    {format!("{}%", *state.progress)}
                </div>
            </div>
        </>
    }
}