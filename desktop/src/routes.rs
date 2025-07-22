use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/modify")]
    Modify,
    #[at("/split")]
    Split,
    #[at("/live")]
    LiveLog,
    #[at("/login")]
    Login,
    #[at("/upload")]
    Upload,
}