use macros::component;
use macros::html;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::EventTarget;
use crate::Result;
use crate::dynui;
use crate::dynui::Element;

#[component]
pub fn Button<'a, F: 'static + FnMut()> (text: &'a str, onclick: F) -> Result<Element> {
    let button = html! {
        <button>{text}</button>
    }?;
    
    let f = JsCast::unchecked_into(Closure::new(onclick).into_js_value());
    EventTarget::add_event_listener_with_callback(&button, "click", &f)?;
    return Ok(button)
}