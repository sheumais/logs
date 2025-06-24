use yew::prelude::*;
use yew_router::prelude::*;
use crate::routes::Route;
use crate::ui::homepage::Homepage;
use crate::ui::live_log::LiveLog;
use crate::ui::modify::ModifyScreen;
use crate::ui::split::SplitCombineScreen;

#[function_component(App)]
pub fn app() -> Html {
        html! {
        <BrowserRouter>
            <Switch<Route> render={Callback::from(switch)} />
        </BrowserRouter>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Modify => html! { <ModifyScreen /> },
        Route::Split => html! { <SplitCombineScreen /> },
        Route::LiveLog => html! { <LiveLog/> },
        _ => html! {<Homepage />}
    }
}