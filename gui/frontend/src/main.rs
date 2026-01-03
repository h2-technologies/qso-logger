use yew::prelude::*;
use yew_router::prelude::*;

mod site;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}


fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <site::home::Home /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component(Main)]
fn app() -> Html {
    html! {
    <>
        <BrowserRouter>
            <Switch<Route> render={switch} /> // <- must be child of <BrowserRouter>
        </BrowserRouter>
    </>
    }
}

fn main() {
    yew::Renderer::<Main>::new().render();
}