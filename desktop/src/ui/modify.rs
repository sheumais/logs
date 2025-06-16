use stylist::css;
use tauri_sys::{core::invoke, event};
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_icons::{Icon, IconId};
use futures::StreamExt;
use crate::routes::Route;

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

    let back_arrow_style = css!(
        r#"
        position: absolute;
        opacity: 0.5;
        top: 0px;
        left: 0px;
        width: 2em;
        height: 2em;
        padding: 0.5em;
        cursor: pointer;
        "#
    );

    let container_style = css!(
        r#"
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 10px;
        left: 50%;
		min-width: 30%;
		padding-top: 1rem;
		position: absolute;
		top: 5%;
		transform: translate(-50%, 0);
		width: min-content;
		text-align: center;
        pointer: default;
    "#
    );

    let icon_wrapper_style = css!(
        r#"
        display: inline-block;
        position: relative;
        "#,
    );

    let icon_border_style = css!(
        r#"
        border: 2px dotted #888;
        border-radius: 8px;
        padding: 0.5em;
        cursor: pointer;
        "#,
    );

    let icon_description = css!(
        r#"
        visibility: hidden;
        opacity: 0;
        background: #222;
        color: #fff;
        text-align: center;
        border-radius: 6px;
        padding: 0.5em 1em;
        position: absolute;
        left: 50%;
        top: 110%;
        transform: translateX(-50%);
        transition: opacity 0.3s;
        pointer-events: none;
        white-space: nowrap;
        font-size: 1em;

        .icon-wrapper:hover & {
            visibility: visible;
            opacity: 1;
        }
        "#,
    );

    html! {
        <>
            <div style={if *is_modifying {"opacity:0;visibility:none;user-select:none;"} else {""}}>
                <Icon class={back_arrow_style} icon_id={IconId::LucideArrowLeftCircle} onclick={go_home} />
            </div>
            <div class={container_style}>
                <div class={classes!(icon_wrapper_style, "icon-wrapper")}>
                    if !*has_chosen_file {
                        <Icon 
                            width={"5em".to_owned()} height={"5em".to_owned()} class={icon_border_style.clone()} icon_id={IconId::LucideUpload} onclick={select_log} 
                        />
                        <div class={icon_description.clone()}>
                            {"Select an encounter log"}
                        </div>
                    }
                </div>
                <div style={if *is_modifying {""} else {"opacity:0;visibility:none;user-select:none;"}}>{format!("{}%", *progress)}</div>
            </div>
        </>
    }
}