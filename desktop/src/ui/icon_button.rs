use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::hooks::use_navigator;

use crate::{routes::Route, ui::style::{back_arrow_style, icon_description, icon_description_visible}};

#[derive(Properties, PartialEq)]
pub struct IconButtonProps {
    pub data: IconData,
    pub description: String,
    #[prop_or_default]
    pub onclick: Option<Callback<MouseEvent>>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or("5em".into())]
    pub width: String,
    #[prop_or("5em".into())]
    pub height: String,
}

#[function_component(IconButton)]
pub fn icon_button(props: &IconButtonProps) -> Html {
    let hovered = use_state(|| false);

    let onmouseover = {
        let hovered = hovered.clone();
        Callback::from(move |_| hovered.set(true))
    };

    let onmouseout = {
        let hovered = hovered.clone();
        Callback::from(move |_| hovered.set(false))
    };

    html! {
        <div class={classes!(props.class.clone())} style="border:none;"
            {onmouseover}
            {onmouseout}
        >
            <Icon
                width={props.width.clone()}
                height={props.height.clone()}
                class={props.class.clone()}
                data={props.data}
                onclick={props.onclick.clone()}
            />
            <div class={if *hovered {
                icon_description_visible()
            } else {
                icon_description()
            }}>
                { &props.description }
            </div>
        </div>
    }
}

#[function_component(BackArrow)]
pub fn back_arrow() -> Html {
    let navigator = use_navigator().unwrap();
    let go_home = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::Home))
    };
    html! {
        <IconButton
            data={IconData::LUCIDE_ARROW_LEFT_CIRCLE}
            description={"Back"}
            onclick={Some(go_home.clone())}
            class={back_arrow_style().clone()}
            width={"2em"}
            height={"2em"}
        />
    }
}