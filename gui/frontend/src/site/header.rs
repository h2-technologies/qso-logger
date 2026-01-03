use yew::prelude::*;

#[function_component(Header)]
pub fn header() -> Html {
    html! {
    <>
        <header>
            <style>{ header_css() }</style>
            <h1>{ "My Yew App" }</h1>
            <nav>
                <a href="/">{ "Home" }</a>
                { " | " }
                <a href="/about">{ "About" }</a>
                { " | " }
                <a href="/contact">{ "Contact" }</a>
                { " | " }
                <a href="/pricing">{ "Pricing" }</a>
            </nav>
        </header>
    </>
    }
}

fn header_css() -> &'static str {
    r#"
    header {
        background-color: #282c34;
        padding: 20px;
        color: white;
        text-align: center;
    }
    nav a {
        color: #61dafb;
        text-decoration: none;
        margin: 0 10px;
    }
    nav a:hover {
        text-decoration: underline;
    }
    "#
}