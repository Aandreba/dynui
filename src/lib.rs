use std::time::Duration;
use js_sys::Function;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Window, Document, HtmlElement};

pub mod component;
pub mod cell;
pub mod html;

pub type Result<T> = ::core::result::Result<T, wasm_bindgen::JsValue>;

pub struct Context {
    pub window: Window,
    pub document: Document,
    pub body: HtmlElement
}

impl Context {
    #[inline]
    pub fn new () -> Option<Self> {
        let window: Window = web_sys::window()?;
        let document = window.document()?;
        let body = document.body()?;
        return Some(Self { window, document, body })
    }

    #[inline]
    pub fn set_timeout<F: 'static + FnOnce()> (&self, f: F) -> Result<i32> {
        let closure = <Function as JsCast>::unchecked_from_js(Closure::once_into_js(f));
        return self.window.set_timeout_with_callback(&closure)
    }

    #[inline]
    pub fn set_interval<F: 'static + FnMut()> (&self, time: Duration, f: F) -> Result<i32> {
        let closure = <Function as JsCast>::unchecked_from_js(Closure::new(f).into_js_value());
        return self.window.set_interval_with_callback_and_timeout_and_arguments_0(&closure, time.as_millis() as i32)
    }
}