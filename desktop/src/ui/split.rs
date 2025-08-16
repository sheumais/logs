use futures::StreamExt;
use stylist::css;
use tauri_sys::{core::invoke, event};
use yew::prelude::*;
use yew_icons::IconId;

use crate::ui::{icon_button::{BackArrow, IconButton}, style::*};

struct SplitCombineState {
    is_splitting: UseStateHandle<bool>,
    is_combining: UseStateHandle<bool>,
    progress: UseStateHandle<u32>,
}

#[function_component(SplitCombineScreen)]
pub fn split_combine_screen() -> Html {
    let is_splitting = use_state(|| false);
    let is_combining = use_state(|| false);
    let progress = use_state(|| 0);
    let state = SplitCombineState {
        is_splitting,
        is_combining,
        progress,
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

    html! {
        <>
            if *state.is_combining || *state.is_splitting { } else { <BackArrow/> }
            <div class={container_style().clone()}>
                <div class={classes!(
                    css!(r#"margin-top: 2em;"#),
                    if *state.is_combining || *state.is_splitting { none_style().clone() } else { hide_style().clone() }
                )}>
                    {format!("{}%", *state.progress)}
                </div>
                <div class={icon_wrapper_style().clone()}>
                    <IconButton
                        icon_id={IconId::BootstrapFileEarmarkArrowUp}
                        description={"Split log"}
                        onclick={Some(split_log.clone())}
                        class={icon_style()}
                    />
                    <IconButton
                        icon_id={IconId::BootstrapFolder}
                        description={"Combine logs"}
                        onclick={Some(combine_log.clone())}
                        class={icon_style()}
                    />
                </div>
            </div>
        </>
    }
}