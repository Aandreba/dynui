#![feature(min_specialization, format_args_nl)]
#![feature(drain_filter)]
#![feature(new_uninit, const_alloc_layout, ptr_metadata, alloc_layout_extra)]

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    }
}

macro_rules! mmod {
    ($($i:ident),+) => {
        $(
            pub mod $i;
        )+
    }
}

use std::{time::Duration, fmt::Arguments};
use dynui::attr::Attribute;
use js_sys::Function;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{Window, Document, HtmlElement};

thread_local! {
    static CONTEXT: Context = Context::new().unwrap();
}

pub extern crate proc as macros;
#[doc(hidden)]
pub extern crate web_sys;
#[doc(hidden)]
pub extern crate into_string;
#[doc(hidden)]
pub(crate) mod dynui { pub use crate::*; }

#[path = "lib/mod.rs"]
pub mod lib;
pub mod component;
pub mod cell;
pub mod attr;

pub type Result<T> = ::core::result::Result<T, wasm_bindgen::JsValue>;

#[derive(Clone)]
pub struct Context {
    pub window: Window,
    pub document: Document,
    pub body: HtmlElement
}

impl Context {
    #[inline]
    fn new () -> Option<Self> {
        console_error_panic_hook::set_once();

        let window: Window = web_sys::window()?;
        let document = window.document()?;
        let body = document.body()?;
        return Some(Self { window, document, body })
    }
}

#[inline(always)]
pub fn context () -> Context {
    CONTEXT.with(Clone::clone)
}

#[inline]
pub fn set_timeout<F: 'static + FnOnce()> (time: Duration, f: F) -> Result<i32> {
    let closure = <Function as JsCast>::unchecked_from_js(Closure::once_into_js(f));
    return CONTEXT.with(|ctx| ctx.window.set_timeout_with_callback_and_timeout_and_arguments_0(&closure, time.as_millis() as i32))
}

#[inline]
pub fn set_interval<F: 'static + FnMut()> (time: Duration, f: F) -> Result<i32> {
    let closure = <Function as JsCast>::unchecked_from_js(Closure::new(f).into_js_value());
    return CONTEXT.with(|ctx| ctx.window.set_interval_with_callback_and_timeout_and_arguments_0(&closure, time.as_millis() as i32))
}

/// Creates a new attribute with the specified name and value
#[inline]
pub fn create_attribute<A: Attribute> (name: &str, value: A) -> Result<web_sys::Attr> {
    let attr: web_sys::Attr = CONTEXT.with(|ctx| ctx.document.create_attribute(name))?;
    value.render(&attr)?;
    Ok(attr)
}

#[inline]
pub fn print (args: Arguments<'_>) {
    #[allow(unused)]
    let s = match args.as_str() {
        Some(s) => JsValue::from_str(s),
        None => JsValue::from_str(&args.to_string())
    };

    #[cfg(target_arch = "wasm32")]
    ::web_sys::console::log_1(&s)
}

#[inline]
pub fn format (args: Arguments<'_>) -> js_sys::JsString {
    #[allow(unused)]
    let s = match args.as_str() {
        Some(s) => JsValue::from_str(s),
        None => JsValue::from_str(&args.to_string())
    };
    return JsCast::unchecked_from_js(s)
}

#[macro_export]
macro_rules! jsprint {
    ($($arg:tt)*) => {
        $crate::print(::std::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! jsprintln {
    ($($arg:tt)*) => {{
        $crate::print(::std::format_args!($($arg)*));
        $crate::print(::std::format_args!("\n"));
    }};
}

#[macro_export]
macro_rules! jsformat {
    ($($arg:tt)*) => {
        $crate::format(::std::format_args!($($arg)*))
    };
}