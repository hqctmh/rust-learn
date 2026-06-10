pub mod app;
pub mod components;
pub mod domain;
pub mod error;
pub mod pages;
pub mod state;

#[cfg(feature = "ssr")]
pub mod api;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;

    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
