#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use crate::app::App;
    yew::Renderer::<App>::new().render();
}

#[cfg(target_arch = "wasm32")]
mod app;
