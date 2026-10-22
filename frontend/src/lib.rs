mod api;
mod components;
mod model;
mod web;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    yew::Renderer::<components::App>::new().render();
    Ok(())
}
