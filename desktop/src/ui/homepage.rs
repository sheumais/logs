use stylist::yew::styled_component;
use tauri_sys::core::invoke;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_icons::IconId;
use crate::{app::{LoginContext, UpdateContext}, routes::Route, ui::{icon_button::IconButton, login::LoginBox, logo::logo, style::*}};

#[derive(Properties, PartialEq)]
pub struct HomepageContainerProps {
    #[prop_or_default]
    pub children: Children,
}

#[styled_component(HomepageContainer)]
fn homepage_container(props: &HomepageContainerProps) -> Html {

    html!{
        <div class={container_style().clone()}>
            <a
                href={"https://github.com/sheumais/logs/"}
                class={logo_style().clone()}
                target="_blank"
                rel="noopener noreferrer"
            >
                {logo()}
            </a>
            <div class={header_style().clone()}>
                {"ESO Log Tool"}
                <div class={subheader_style().clone()}>
                    {format!("v{}", env!("CARGO_PKG_VERSION"))}
                </div>
            </div>
            {for props.children.iter()}
        </div>
    }
}

#[function_component(Homepage)]
pub fn homepage() -> Html {
    let navigator = use_navigator().unwrap();
    let modify_log = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Modify);
        })
    };
    let split_log = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Split);
        })
    };
    let live_log = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::LiveLog);
        })
    };
    let upload = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Upload);
        })
    };
    let terms = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Terms);
        })
    };

    let update = Callback::from(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            invoke::<()>("download_and_install_update", &()).await;
        });
    });

    let login_ctx = use_context::<LoginContext>().expect("LoginContext not found");
    let update_ctx = use_context::<UpdateContext>().expect("UpdateContext not found");

    html! {
        <>
            <LoginBox/>
            <HomepageContainer>
                <div class={icon_wrapper_style()}>
                    if login_ctx.is_some() {
                        <IconButton
                            icon_id={IconId::LucideUpload}
                            description={"Upload to esologs.com"}
                            onclick={Some(upload.clone())}
                            class={icon_style()}
                        />
                    } else {
                        <IconButton
                            icon_id={IconId::BootstrapFileEarmarkBreak}
                            description={"Scan encounter log"}
                            onclick={Some(modify_log.clone())}
                            class={icon_style()}
                        />
                        <IconButton
                            icon_id={IconId::BootstrapFolderSymlink}
                            description={"Live log with scan"}
                            onclick={Some(live_log.clone())}
                            class={icon_style()}
                        />
                    }
                    <IconButton
                        icon_id={IconId::BootstrapFiles}
                        description={"Split/Combine logs"}
                        onclick={Some(split_log.clone())}
                        class={icon_style()}
                    />
                </div>
            </HomepageContainer>
            <div onclick={terms.clone()} class={text_link_style()} style={"position:fixed;bottom:0px;left:0px;padding:0.5em;font-size:1em;"}>
                {"Terms"}
            </div>
            if let Some(update_info) = &*update_ctx {
                <div onclick={update.clone()} style={"position:fixed;bottom:0px;width:30%;left:35%;padding:0.5em;text-align:center;cursor:pointer;"}>
                    {format!("Update available: {}", update_info.version)}
                </div>
            }
        </>
    }
}