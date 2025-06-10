use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use gloo_utils::format::JsValueSerdeExt;
use parser::effect::{self, Effect};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[function_component(App)]
pub fn app() -> Html {
    let output = use_state(|| Vec::<String>::new());
    let effect_output = use_state(|| Vec::<Effect>::new());

    let on_query_fights = {
        let output = output.clone();
        Callback::from(move |_| {
            let output = output.clone();

            spawn_local(async move {
                let args = JsValue::NULL;
                let result = invoke("query_fights", args).await;

                match result {
                    js_value => {
                        let lines: Result<Vec<String>, _> = js_value.into_serde();
                        match lines {
                            Ok(lines) => output.set(lines),
                            Err(_) => output.set(vec!["Failed to parse result from query_fights".to_string()]),
                        }
                    }
                }
            });
        })
    };

    let on_query_effects = {
        let effect_output = effect_output.clone();
        Callback::from(move |_| {
            let effect_output = effect_output.clone();

            spawn_local(async move {
                let args = JsValue::NULL;
                let result = invoke("query_effects", args).await;

                match result {
                    js_value => {
                        let effects: Result<Vec<Effect>, _> = js_value.into_serde();
                        match effects {
                            Ok(effects) => effect_output.set(effects),
                            Err(_) => effect_output.set(vec![]),
                        }
                    }
                }
            });
        })
    };

    fn get_existing_or_default(path: &str, default: &str) -> String {
        if std::path::Path::new(path).exists() {
            path.to_string()
        } else {
            default.to_string()
        }
    }
    
    html! {
        <div>
            <button onclick={on_query_fights}>{"Query Fights"}</button>
            <div>
                { for output.iter().map(|line| html! { <p>{ line }</p> }) }
            </div>
            <div>
                <button onclick={on_query_effects}>{"Query Effects"}</button>
                <table>
                    <thead>
                        <tr>
                            <th>{"Icon"}</th>
                            <th>{"Name"}</th>
                            <th>{"Id"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for effect_output.iter().map(|effect| {
                            let icon_path = format!("public/icons/{}", effect.ability.icon);
                            let default_icon = "public/icons/ability_mage_065.png";

                            html! {
                                <tr>
                                    <td>
                                        <img
                                            src={icon_path.clone()}
                                            width="50"
                                            height="50"
                                        />
                                    </td>
                                    <td>{ &effect.ability.name }</td>
                                    <td>{ &effect.ability.id }</td>
                                </tr>
                            }
                        }) }
                    </tbody>
                </table>
            </div>
        </div>
    }
}