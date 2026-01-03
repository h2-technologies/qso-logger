use yew::prelude::*;

use crate::site::header;

#[function_component(Home)]
pub fn home() -> Html {
    html! {
    <>
        <header::Header />
        <div>
            <style>{ home_css() }</style>
            <h2>{ "Welcome to the Home Page" }</h2>
            <p>{ "This is the main landing page of the application." }</p>
        </div>
    </>
    }
}

fn home_css() -> &'static str {
    r#"
    div {
        padding: 20px;
    }
    h2 {
        color: #333;
    }
    p {
        font-size: 16px;
        color: #666;
    }
    "#
}