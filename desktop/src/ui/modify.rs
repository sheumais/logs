use tauri_sys::{core::invoke, event};
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_icons::{Icon, IconId};
use futures::StreamExt;
use crate::{routes::Route, ui::style::*};

#[function_component(ModifyScreen)]
pub fn modify_screen() -> Html {
    let navigator = use_navigator().unwrap();
    let go_home = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Home);
        })
    };

    let is_modifying = use_state(|| false);
    let progress = use_state(|| 0u32);
    let has_chosen_file = use_state(|| false);
    let progress_effect = progress.clone();
    let is_modifying_effect = is_modifying.clone();
    let navigator_effect = navigator.clone();

    use_effect(move || {
        let progress = progress_effect.clone();
        let is_modifying = is_modifying_effect.clone();
        let navigator = navigator_effect.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(mut events) = event::listen::<u32>("log_modify_progress").await {
                    while let Some(e) = events.next().await {
                        progress.set(e.payload);
                        if e.payload >= 100 {
                            is_modifying.set(false);
                            navigator.push(&Route::Home);
                        }
                    }
                }
            });
        || ()
    });

    let select_log = {
        let has_chosen_file = has_chosen_file.clone();
        let is_modifying = is_modifying.clone();
        move |_| {
            let has_chosen_file = has_chosen_file.clone();
            let is_modifying = is_modifying.clone();
            wasm_bindgen_futures::spawn_local(async move {
                invoke::<()>("pick_and_load_file", &()).await;
                has_chosen_file.set(true);
                is_modifying.set(true);

                invoke::<()>("modify_log_file", &()).await;
            });
        }
    };

    html! {
        <>
            <div class={classes!(if *is_modifying {hide_style().clone()} else {none_style().clone()})}>
                <Icon class={back_arrow_style().clone()} icon_id={IconId::LucideArrowLeftCircle} onclick={go_home} />
            </div>
            <div class={container_style().clone()}>
                <div class={classes!(icon_wrapper_style().clone(), "icon-wrapper")}>
                    if !*has_chosen_file {
                        <Icon 
                            width={"5em".to_owned()} height={"5em".to_owned()} class={icon_border_style().clone()} icon_id={IconId::LucideUpload} onclick={select_log} 
                        />
                        <div class={icon_description().clone()}>
                            {"Select an encounter log"}
                        </div>
                    }
                </div>
                <div class={
                    if *is_modifying {
                        classes!(header_style().clone())
                    } else {
                        classes!(hide_style().clone(), header_style().clone())
                    }
                }>{format!("{}%", *progress)}</div>
            </div>
        </>
    }
}