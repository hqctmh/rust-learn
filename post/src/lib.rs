#![recursion_limit = "256"]

pub mod app;
pub mod components;
pub mod domain;
pub mod error;
#[cfg(feature = "ssr")]
pub mod object_store;
pub mod page_data;
pub mod pages;
#[cfg(feature = "ssr")]
pub mod repositories;
pub mod services;
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
