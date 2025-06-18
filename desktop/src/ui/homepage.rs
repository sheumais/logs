use stylist::yew::styled_component;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_icons::{Icon, IconId};
use crate::{routes::Route, ui::{logo::logo, style::{container_style, header_style, icon_description, icon_description_visible, icon_style, icon_style_inactive, icon_wrapper_style, logo_style, subheader_style}}};

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
    let hovered = use_state(|| None::<usize>);
    
    let buttons = vec![
        (
            IconId::BootstrapFileEarmarkBreak,
            icon_style().clone(),
            Some(modify_log.clone()),
            "Scan encounter log",
        ),
        (
            IconId::BootstrapFileEarmarkBarGraph,
            icon_style_inactive().clone(),
            Some(Callback::noop()),
            "Log analysis (coming soon)",
        ),
        (
            IconId::BootstrapFiles,
            icon_style().clone(),
            Some(split_log.clone()),
            "Split/Combine logs",
        ),
        (
            IconId::BootstrapFolderSymlink,
            icon_style_inactive().clone(),
            Some(Callback::noop()),
            "Live log with scan (coming soon)",
        ),
    ];

    html! {
        <>
            <HomepageContainer>
                <div class={icon_wrapper_style().clone()}>
                    { for buttons.iter().enumerate().map(|(i, (icon_id, style, onclick, desc))| {
                        let hovered = hovered.clone();
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
                                <Icon width={"5em".to_owned()} height={"5em".to_owned()} class={style.clone()} icon_id={icon_id.clone()} onclick={onclick.clone()} />
                                <div class={if hovered.as_ref() == Some(&i) { icon_description_visible().clone() } else { icon_description().clone() }}>
                                    {desc}
                                </div>
                            </div>
                        }
                    })}
                </div>
            </HomepageContainer>
        </>
    }
}