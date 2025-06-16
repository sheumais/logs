use stylist::{css, yew::styled_component};
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_icons::{Icon, IconId};
use crate::{routes::Route, ui::logo::logo};

#[derive(Properties, PartialEq)]
pub struct HomepageContainerProps {
    #[prop_or_default]
    pub children: Children,
}

#[styled_component(HomepageContainer)]
fn homepage_container(props: &HomepageContainerProps) -> Html {
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
    "#
    );
    let logo_style = css!(
        r#"
        width: 60%;
        "#
    );
    let header_style = css!(
        r#"
        position: relative;
        display: inline-block;
        font-size: 24px;
        color: white;
        font-weight: bold;
        margin: 0;
        text-align: center;
        user-select: none;
        margin-bottom: 30px;
        "#
    );

    let subheader_style = css!(
        r#"
        position: absolute;
        top: 2px;
        right: -2em;
        font-size: 16px;
        color: #777;
        font-weight: normal;
        "#
    );


    html!{
        <div class={container_style}>
            <a
                href={"https://github.com/sheumais/logs/"}
                class={logo_style}
                target="_blank"
                rel="noopener noreferrer"
            >
                {logo()}
            </a>
            <div class={header_style}>
                {"ESO Log Tool"}
                <div class={subheader_style}>
                    {"v0.1"}
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
    let hovered = use_state(|| None::<usize>);
    
    let icon_wrapper_style = css!(
        r#"
        display: inline-block;
        position: relative;
        "#,
    );

    let icon_style = css!(
        r#"
        display: inline-block;
        position: relative;
        margin: 0 1em;
        cursor: pointer;
        "#,
    );

    let icon_style_inactive = css!(
        r#"
        display: inline-block;
        position: relative;
        margin: 0 1em;
        opacity: 0.5;
        cursor: not-allowed;
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
        user-select: none;
        "#,
    );

    let icon_description_visible = css!(
        r#"
        visibility: visible;
        opacity: 1;
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
        user-select: none;
        "#,
    );

    let buttons = vec![
        (
            IconId::BootstrapFileEarmarkBreak,
            icon_style.clone(),
            Some(modify_log.clone()),
            "Scan encounter log",
        ),
        (
            IconId::BootstrapClipboardData,
            icon_style_inactive.clone(),
            None,
            "Log analysis (coming soon)",
        ),
    ];

    html! {
        <>
            <HomepageContainer>
                <div class={icon_wrapper_style}>
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
                                <div class={if hovered.as_ref() == Some(&i) { icon_description_visible.clone() } else { icon_description.clone() }}>
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