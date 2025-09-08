use std::rc::Rc;
use esologtool_common::LoginResponse;
use stylist::{css, Style};
use tauri_sys::core::{invoke, invoke_result};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_icons::{Icon, IconId};
use crate::{app::LoginContext, routes::Route, ui::{icon_button::{BackArrow, IconButton}, style::*}};

#[function_component(LoginBox)]
pub fn login_component() -> Html {
    let login_ctx = use_context::<LoginContext>().expect("LoginContext not found");
    let navigator = use_navigator().unwrap();
    let go_to_login = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Login);
        })
    };

    {
        let login_ctx = login_ctx.clone();
        use_effect(move || {
            wasm_bindgen_futures::spawn_local(async move {
                match invoke::<Option<LoginResponse>>(
                    "get_saved_login_response",
                    &serde_json::json!({})
                ).await {
                    Some(login) => login_ctx.set(Some(Rc::new(login))),
                    None => login_ctx.set(None),
                }
            });
            || ()
        });
    }

    html! {
        <div class={login_box_style().clone()} onclick={go_to_login.clone()}>
            if let Some(login) = &*login_ctx {
                <span class={classes!("login-name", custom_user_style(&login.user.username))}>{ login.user.username.clone() }</span>
            } else {
                <span class="login-name">{ "Login" }</span>
            }
            <Icon class={icon_style_small().clone()} icon_id={IconId::BootstrapPersonCircle} />
        </div>
    }
}

#[function_component(LoginScreen)]
pub fn login_screen() -> Html {
    let username = use_state(|| String::new());
    let password = use_state(|| String::new());
    let logging_in = use_state(|| false);
    let error = use_state(|| None as Option<String>);

    let login_ctx = use_context::<LoginContext>().expect("LoginContext not found");

    {
        let login_ctx = login_ctx.clone();
        use_effect(move || {
            wasm_bindgen_futures::spawn_local(async move {
                match invoke::<Option<LoginResponse>>(
                    "get_saved_login_response",
                    &serde_json::json!({})
                ).await {
                    Some(login) => login_ctx.set(Some(Rc::new(login))),
                    None => login_ctx.set(None),
                }
            });
            || ()
        });
    }

    let on_submit = {
        let username = username.clone();
        let password = password.clone();
        let login_ctx = login_ctx.clone();
        let logging_in = logging_in.clone();
        let error = error.clone();
        move |event: SubmitEvent| {
            event.prevent_default();
            let username = username.clone();
            let password = password.clone();
            let login_ctx = login_ctx.clone();
            let logging_in = logging_in.clone();
            let error = error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if *logging_in == true {
                    return;
                } else if *username == "" || *password == "" {
                    error.set(Some("Email and password cannot be empty".to_string()));
                    return;
                } {
                    logging_in.set(true);
                    error.set(None);
                    match invoke_result::<LoginResponse, String>(
                        "login",
                        &serde_json::json!({
                            "username": *username,
                            "password": *password,
                        }),
                    )
                    .await
                    {
                        Ok(body) => {
                            login_ctx.set(Some(Rc::new(body)));
                            error.set(None);
                        }
                        Err(other) => {
                            error.set(Some(other.to_string()));
                        }
                    }
                    logging_in.set(false);
                }
            });
        }
    };

    let logout = {
        let login_ctx = login_ctx.clone();
        move |_| {
            let login_ctx = login_ctx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                invoke::<()>("logout", &()).await;
                login_ctx.set(None)
            });
        }
    };

    let form_style = Style::new(css!(r#"
            display: flex;
            flex-direction: column;
            align-items: center;
        "#
    )).expect("Error creating form style");

    let input_container_style = Style::new(css!(r#"
            display: flex;
            flex-direction: column;
            max-width: 33%;
            white-space: nowrap;
            margin-bottom: 0.75em;
        "#
    )).expect("Error creating input container style");

    let input_title_style = Style::new(css!(r#"
            font-weight: bold;
            margin-bottom: 0.5em;
            width: 40%;
        "#
    )).expect("Error creating input title style");

    let input_style = Style::new(css!(r#"
            padding: 0.5em;
            margin-bottom: 1em;
            border-radius: 4px;
            border: 0px solid transparent;
            outline: none;
        "#
    )).expect("Error creating input style");

    let input_button_style = Style::new(css!(r#"
            padding: 0.5em 1em;
            color: white;
            border: none;
            border-radius: 25px;
            cursor: pointer;
            width: 25%;
            min-width: min-content;
            text-align: center;
            transition: width 0.3s, background 0.3s, border-radius 0.3s;
            background: rgba(0,0,0,0.2);

            &:hover {
                background: rgba(0,0,0,0.5);
                border-radius: 10px;
            }
        "#
    )).expect("Error creating input button style");

    html! {
        <>
            <BackArrow/>
            <div class={container_style().clone()}>
                if let Some(login) = &*login_ctx {
                    <div class={input_title_style.clone()}>
                        { format!("Welcome, {}! You have successfully logged in.", login.user.username) }
                    </div>
                    <IconButton
                        icon_id={IconId::LucideLogOut}
                        description={"Logout"}
                        onclick={Some(logout.clone())}
                        class={icon_style()}
                        width={"2em"}
                        height={"2em"}
                    />
                    if let Some(err) = &*error {
                        <div style="color: red; margin-bottom: 1em;">{ err }</div>
                    }
                } else {
                    <form class={form_style.clone()} onsubmit={on_submit} autocomplete="off">
                        <div class={header_style().clone()}>
                            { "Log in using esologs.com credentials"}
                        </div>
                        <div class={paragraph_style().clone()}>
                            { "Logging in allows uploading logs directly to esologs.com and is NOT mandatory. Please note that this application is completely unaffiliated and not endorsed by esologs.com. You submit your details here in full knowledge of the terms of the software." }
                        </div>
                        <div class={paragraph_style().clone()}>
                            { "Your details are sent directly to esologs.com only and are not saved. Cookies set by esologs.com and your profile information are stored locally until you log out." }
                        </div>
                        <div class={paragraph_style().clone()}>
                            {"If you have any concerns you are welcome to audit the code, build it from" }
                            <span>
                                <a class={fancy_link_style().clone()} href={"https://github.com/sheumais/logs/"} target="_blank" rel="noopener noreferrer">
                                    {"source"}
                                </a>
                            </span>
                            {"yourself, or contact me through"}
                            <span>
                                <a class={fancy_link_style().clone()} href={"https://discord.gg/FjJjXHjUQ4"} target="_blank" rel="noopener noreferrer">
                                    {"Discord"}
                                </a>
                            </span>
                            {"."}
                        </div>
                        if let Some(err) = &*error {
                            <div style="color: red; margin-bottom: 1em;">{ err }</div>
                        }
                        <div class={input_container_style.clone()}>
                            <div class={input_title_style.clone()}>
                                { "Email Address:" }
                            </div>
                            <input
                                class={input_style.clone()}
                                name="email"
                                autocomplete="off"
                                type="text"
                                placeholder="email"
                                value={(*username).clone()}
                                oninput={Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    username.set(input.value());
                                })}
                            />
                            <div class={input_title_style.clone()}>
                                { "Password:" }
                            </div>
                            <input
                                class={input_style.clone()}
                                name="password"
                                autocomplete="off"
                                type="password"
                                placeholder="password"
                                value={(*password).clone()}
                                oninput={Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    password.set(input.value());
                                })}
                            />
                        </div>
                        <button class={classes!(if *logging_in {hide_style().clone()} else {input_button_style.clone()})} type="submit">
                            { "Login" }
                        </button>
                    </form>
                }
            </div>
        </>
    }
}