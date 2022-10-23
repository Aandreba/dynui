//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use std::time::Duration;

use dynui::{Context, cell::Cell, Result};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() -> Result<()> {
    console_error_panic_hook::set_once();
    let ctx = Context::new().unwrap();

    let mut cell = Cell::new(1u32)?;
    ctx.body.append_child(&cell)?;

    ctx.set_interval(Duration::from_secs(1), move || {
        cell.update(|x| x + 1);
    });

    Ok(())
}