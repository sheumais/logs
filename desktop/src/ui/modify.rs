use tauri_sys::{core::invoke, event};
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_icons::IconId;
use futures::StreamExt;
use crate::{routes::Route, ui::{icon_button::{BackArrow, IconButton}, style::*}};

#[function_component(ModifyScreen)]
pub fn modify_screen() -> Html {
    let navigator = use_navigator().unwrap();
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
                <BackArrow/>
            </div>
            <div class={container_style().clone()}>
                if !*has_chosen_file {
                    <IconButton
                        icon_id={IconId::LucideUpload}
                        description={"Select an encounter log"}
                        onclick={Some(select_log.clone())}
                        class={icon_border_style()}
                    />
                    <div class={paragraph_style()}>
                        {"This will create a new, modified log file with fixes applied. This can then be uploaded using the official esologs.com uploader."}
                    </div>
                    <div class={paragraph_style()}>
                        {"If you intended to upload a log directly to esologs.com, please press the back arrow and log in instead."}
                    </div>
                }
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