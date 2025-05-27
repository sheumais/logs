use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use gloo_utils::format::JsValueSerdeExt;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[function_component(App)]
pub fn app() -> Html {
    let output = use_state(|| Vec::<String>::new());

    let on_query_click = {
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

    html! {
        <div>
            <button onclick={on_query_click}>{"Query Fights"}</button>
            <div>
                { for output.iter().map(|line| html! { <p>{ line }</p> }) }
            </div>
        </div>
    }
}