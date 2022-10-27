use std::fmt::Display;
use into_string::FromString;
use into_string::IntoCowStr;
use macros::component;
use macros::html;
use wasm_bindgen::JsCast;
use crate::Result;
use crate::cell::CellLike;
use crate::cell::MutableCell;
use crate::cell::SharedCell;
use crate::dynui;
use crate::dynui::Element;

#[component]
pub fn Input<'a, V: 'static + MutableCell> (ty: &'a str, mut value: V) -> Result<Element> 
where
    <V as CellLike>::Value: FromString,
    <<V as CellLike>::Value as FromString>::Err: Display
{
    let element = html! {
        <input r#type={ty} />
    }?;

    let my_element: web_sys::HtmlInputElement = JsCast::unchecked_into(element.0.clone());
    element.add_listener("keyup", move |_| {
        let text = my_element.value();
        let v = match <V::Value as FromString>::from_string(text) {
            Ok(x) => x,
            Err(e) => wasm_bindgen::throw_str(&e.into_cow_str())
        };
        value.set(v)
    })?;

    return Ok(element)
}

#[component]
pub fn Button<'a, F: 'static + FnMut(web_sys::Event)> (text: &'a str, default: bool, mut onclick: F) -> Result<Element> {
    let button = html! {
        <button>{text}</button>
    }?;
    
    match default {
        false => button.add_listener("click", move |e: web_sys::Event| {
            e.prevent_default();
            onclick(e)
        }),
        true => button.add_listener("click", onclick)
    }?;

    return Ok(button)
}