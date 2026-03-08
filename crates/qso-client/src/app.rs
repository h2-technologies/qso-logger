use qso_core::ipv6;
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    let callsign = use_state(String::new);
    let server_url = use_state(|| "http://localhost:8080".to_string());
    let ipv6_address = use_state(String::new);
    let status = use_state(String::new);

    let on_callsign_input = {
        let callsign = callsign.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            callsign.set(input.value());
        })
    };

    let on_server_url_input = {
        let server_url = server_url.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            server_url.set(input.value());
        })
    };

    let on_generate = {
        let callsign = callsign.clone();
        let ipv6_address = ipv6_address.clone();
        Callback::from(move |_| {
            let cs = callsign.to_uppercase();
            if cs.is_empty() {
                return;
            }
            let addr = ipv6::generate_ipv6_address(&cs, 0);
            ipv6_address.set(addr.to_string());
        })
    };

    let on_register = {
        let callsign = callsign.clone();
        let server_url = server_url.clone();
        let ipv6_address = ipv6_address.clone();
        let status = status.clone();
        Callback::from(move |_| {
            let cs = (*callsign).clone().to_uppercase();
            let url = format!("{}/register", *server_url);
            let addr = (*ipv6_address).clone();
            let status = status.clone();

            if cs.is_empty() || addr.is_empty() {
                status
                    .set("Please enter a callsign and generate an IPv6 address first.".to_string());
                return;
            }

            let body = serde_json::json!({
                "callsign": cs,
                "ipv6_address": addr,
                "subnet": 0,
                "tcp_port": 7300u16,
            });

            wasm_bindgen_futures::spawn_local(async move {
                match gloo_net::http::Request::post(&url)
                    .json(&body)
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) => {
                        let text = resp.text().await.unwrap_or_default();
                        status.set(format!("Response: {}", text));
                    }
                    Err(e) => {
                        status.set(format!("Error: {}", e));
                    }
                }
            });
        })
    };

    html! {
        <div class="container">
            <h1>{"QSO Logger - IPv6 Registration"}</h1>
            <div>
                <label>{"Callsign: "}
                    <input
                        type="text"
                        value={(*callsign).clone()}
                        oninput={on_callsign_input}
                        placeholder="e.g. W1AW"
                    />
                </label>
            </div>
            <div>
                <label>{"Server URL: "}
                    <input
                        type="text"
                        value={(*server_url).clone()}
                        oninput={on_server_url_input}
                    />
                </label>
            </div>
            <div>
                <button onclick={on_generate}>{"Generate IPv6 Address"}</button>
            </div>
            if !(*ipv6_address).is_empty() {
                <div>
                    <strong>{"IPv6 Address: "}</strong>
                    <span>{(*ipv6_address).clone()}</span>
                </div>
            }
            <div>
                <button onclick={on_register}>{"Register"}</button>
            </div>
            if !(*status).is_empty() {
                <div>
                    <strong>{"Status: "}</strong>
                    <span>{(*status).clone()}</span>
                </div>
            }
        </div>
    }
}
