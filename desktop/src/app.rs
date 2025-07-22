use std::rc::Rc;

use cli::esologs_format::{LoginResponse, UploadSettings};
use yew::prelude::*;
use yew_router::prelude::*;
use crate::routes::Route;
use crate::ui::homepage::Homepage;
use crate::ui::live_log::LiveLog;
use crate::ui::login::LoginScreen;
use crate::ui::modify::ModifyScreen;
use crate::ui::split::SplitCombineScreen;
use crate::ui::upload::UploadScreen;

pub type LoginContext = UseStateHandle<Option<Rc<LoginResponse>>>;
pub type ESOLogUploadSettings = UseStateHandle<Option<Rc<UploadSettings>>>;

#[function_component(App)]
pub fn app() -> Html {
    let login_state = use_state(|| None::<Rc<LoginResponse>>);
    let upload_settings = use_state(|| None::<Rc<UploadSettings>>);

    html! {
        <ContextProvider<LoginContext> context={login_state}>
            <ContextProvider<ESOLogUploadSettings> context={upload_settings}>
                <BrowserRouter>
                    <Switch<Route> render={Callback::from(switch)} />
                </BrowserRouter>
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
        _ => html! {<Homepage />}
    }
}