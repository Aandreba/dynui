use std::time::Duration;

use js_sys::{Promise, Function};
use macros::{component};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::Node;
use crate::cell::{Cell, MutableCell};
use crate::component::Component;
use crate::{dynui, Result, CONTEXT};

#[component]
pub fn Future<Fut: 'static + std::future::Future, P: Component> (fut: Fut, placeholder: P) -> Result<Node> where Fut::Output: Component {
    let mut my_element: Cell<Node> = Cell::new(placeholder.render()?);
    let element = my_element.render()?;

    wasm_bindgen_futures::spawn_local(async move {
        match fut.await.render() {
            Ok(x) => my_element.set(x),
            Err(e) => wasm_bindgen::throw_val(e)
        }
    });

    return Ok(element)
}

#[inline]
pub fn sleep (dur: Duration) -> JsFuture {
    return wasm_bindgen_futures::JsFuture::from(sleep_promise(dur))
}

pub fn sleep_promise (dur: Duration) -> Promise {
    let mut f = |resolve: Function, reject: Function| {
        match CONTEXT.with(|ctx| 
            ctx.window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, dur.as_millis() as i32)
        ) {
            Ok(_) => {},
            Err(e) => match reject.call1(&JsValue::UNDEFINED, &e) {
                Ok(_) => {},
                Err(e) => wasm_bindgen::throw_val(e)
            }
        }
    };

    return Promise::new(&mut f);
}