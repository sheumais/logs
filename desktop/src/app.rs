use std::rc::Rc;

use esologtool_common::{LoginResponse, UpdateInformation, UploadSettings};
use futures::StreamExt;
use tauri_sys::event::listen;
use yew::prelude::*;
use yew_router::prelude::*;
use crate::routes::Route;
use crate::ui::homepage::Homepage;
use crate::ui::icon_button::BackArrow;
use crate::ui::live_log::LiveLog;
use crate::ui::login::LoginScreen;
use crate::ui::modify::ModifyScreen;
use crate::ui::split::SplitCombineScreen;
use crate::ui::terms::TermsComponent;
use crate::ui::upload::UploadScreen;

pub type LoginContext = UseStateHandle<Option<Rc<LoginResponse>>>;
pub type ESOLogUploadSettings = UseStateHandle<Option<Rc<UploadSettings>>>;
pub type UpdateContext = UseStateHandle<Option<Rc<UpdateInformation>>>;

#[function_component(App)]
pub fn app() -> Html {
    let login_state = use_state(|| None::<Rc<LoginResponse>>);
    let upload_settings = use_state(|| None::<Rc<UploadSettings>>);
    let update_state = use_state(|| None::<Rc<UpdateInformation>>);
    let update_state_effect = update_state.clone();
    
    use_effect(move || {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(mut version) = listen::<UpdateInformation>("update-available").await {
                while let Some(e) = version.next().await {
                    update_state_effect.set(Some(Rc::new(e.payload)));
                }
            }
        });
        || ()
    });

    html! {
        <ContextProvider<LoginContext> context={login_state}>
            <ContextProvider<ESOLogUploadSettings> context={upload_settings}>
                <ContextProvider<UpdateContext> context={update_state}>
                    <BrowserRouter>
                        <Switch<Route> render={Callback::from(switch)} />
                    </BrowserRouter>
                </ContextProvider<UpdateContext>>
            </ContextProvider<ESOLogUploadSettings>>
        </ContextProvider<LoginContext>>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Modify => html! { <ModifyScreen/> },
        Route::Split => html! { <SplitCombineScreen/> },
        Route::LiveLog => html! { <LiveLog/> },
        Route::Login => html! { <LoginScreen/> },
        Route::Upload => html! { <UploadScreen/> },
        Route::Terms => html! { <> <BackArrow/> <TermsComponent/> </>},
        _ => html! {<Homepage />}
    }
}