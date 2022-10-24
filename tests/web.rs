#![feature(type_name_of_val)]
//! Test suite for the Web and headless browsers.

//#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use std::{time::Duration, ops::AddAssign};

use dynui::{*, macros::*, cell::{Cell, SharedCell}, jsprintln};
use wasm_bindgen_test::*;
use web_sys::Element;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() -> Result<()> {
    let mut cell = Cell::new(1u32)?;
    context().body.append_child(&cell)?;

    set_interval(Duration::from_secs(1), move || {
        cell.mutate(|x| x.add_assign(1));
    })?;

    Ok(())
}

#[wasm_bindgen_test]
fn html () -> Result<()> {
    use dynui::lib::Button;
    use dynui::component::Component;

    let value = SharedCell::new(0u128)?;
    let my_value = value.clone();
    let onclick = move || my_value.mutate(|x| x.add_assign(1));

    let alpha = html! {
        <span>{"Clicked "}{value}{" times!"}</span>
        <Button text={"Click me!"} onclick={onclick} />
    }?;

    let alpha = alpha.render()?;
    context().body.append_child(&alpha)?;
    Ok(())
}