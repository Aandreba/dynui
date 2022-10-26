use std::time::Duration;
use js_sys::{Promise, Function};
use macros::{component};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use crate::component::{Component, Node};
use crate::{dynui, Result, CONTEXT, jseprintln};

#[component]
pub fn Future<Fut: 'static + std::future::Future, P: Component> (fut: Fut, placeholder: P) -> Result<Node> where Fut::Output: Component {
    let element = placeholder.render()?;
    let my_element = element.0.clone();

    wasm_bindgen_futures::spawn_local(async move {
        match fut.await.render() {
            Ok(x) => match my_element.parent_node() {
                Some(parent) => match parent.replace_child(&x.0, &my_element) {
                    Ok(_) => {},
                    Err(e) => wasm_bindgen::throw_val(e)
                },
                None => {
                    #[cfg(debug_assertions)]
                    jseprintln!("previous node doesn't have a parent")
                }
            },

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