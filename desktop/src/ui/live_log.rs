use futures::StreamExt;
use tauri_sys::{core::invoke, event};
use yew::{classes, function_component, html, use_effect, use_state, Callback, Html, UseStateHandle};
use yew_icons::IconId;
use crate::ui::icon_button::{BackArrow, IconButton};
use crate::ui::style::*;

struct LiveLogState {
    live_logging: UseStateHandle<bool>,
    progress: UseStateHandle<u32>,
}

#[function_component(LiveLog)]
pub fn live_log() -> Html {
    let live_logging = use_state(|| false);
    let progress = use_state(|| 0);
    let state = LiveLogState {
        live_logging,
        progress,
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

    html! {
        <>
            <BackArrow/>
            <div class={classes!(container_style().clone())}>
                <div class={classes!(
                    if *state.live_logging { none_style().clone() } else { hide_style().clone() }
                )}>
                    {format!("{} new lines written", *state.progress)}
                </div>
                <div class={icon_wrapper_style().clone()}>
                    <IconButton
                        icon_id={IconId::BootstrapFolderSymlink}
                        description={"Live log"}
                        onclick={Some(live_log.clone())}
                        class={icon_border_style().clone()}
                    />
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